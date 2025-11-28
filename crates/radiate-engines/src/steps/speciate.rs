use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Executor, Genotype, MetricSet, Objective, Species, diversity::Distance,
    metric_names,
};
use radiate_error::Result;
use std::sync::{Arc, Mutex, RwLock};

pub struct SpeciateStep<C>
where
    C: Chromosome,
{
    pub(crate) threashold: f32,
    pub(crate) objective: Objective,
    pub(crate) diversity: Arc<dyn Distance<Genotype<C>>>,
    pub(crate) executor: Arc<Executor>,
}

impl<C> SpeciateStep<C>
where
    C: Chromosome + 'static,
{
    /// Build chunked populations and the closures to run for each chunk.
    /// Returns:
    /// - chunked_members: Vec of Arc<RwLock<Vec<(idx, genotype)>>>
    /// - batches: Vec of closures to submit to the executor
    fn build_chunks_and_batches(
        &self,
        ecosystem: &mut Ecosystem<C>,
        mascots: Arc<Vec<Genotype<C>>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
    ) -> Result<()> {
        let pop_len = ecosystem.population().len();
        let num_threads = self.executor.num_workers().max(1);
        let chunk_size = (pop_len as f32 / num_threads as f32).ceil() as usize;

        let mut chunked_members = Vec::new();
        let mut batches = Vec::new();

        for chunk_start in (0..pop_len).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(pop_len);

            let chunk_population = Arc::new(RwLock::new(
                ecosystem
                    .population_mut()
                    .iter_mut()
                    .enumerate()
                    .skip(chunk_start)
                    .take(chunk_end)
                    .try_fold(Vec::new(), |mut acc, (idx, pheno)| {
                        let genotype = pheno.take_genotype()?;
                        acc.push((idx, genotype));
                        Ok::<_, radiate_core::RadiateError>(acc)
                    })?,
            ));

            let threshold = self.threashold;
            let diversity = Arc::clone(&self.diversity);
            let assignments = Arc::clone(&assignments);
            let distances = Arc::clone(&distances);
            let population = Arc::clone(&chunk_population);
            let species_snapshot = Arc::clone(&mascots);

            batches.push(move || {
                Self::process_chunk(
                    population,
                    species_snapshot,
                    threshold,
                    diversity,
                    assignments,
                    distances,
                );
            });

            chunked_members.push(chunk_population);
        }

        self.executor.submit_blocking(batches);

        self.restore_genotypes(ecosystem, chunked_members);

        Ok(())
    }

    #[inline]
    fn restore_genotypes(
        &self,
        ecosystem: &mut Ecosystem<C>,
        chunked_members: Vec<Arc<RwLock<Vec<(usize, Genotype<C>)>>>>,
    ) {
        for chunks in chunked_members {
            let mut chunks = chunks.write().unwrap();
            let mut taken_genotypes = Vec::with_capacity(chunks.len());
            std::mem::swap(&mut *chunks, &mut taken_genotypes);

            for (idx, geno) in taken_genotypes {
                ecosystem.get_phenotype_mut(idx).unwrap().set_genotype(geno);
            }
        }
    }

    #[inline]
    fn assign_unassigned(
        &self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        assignments: &[Option<usize>],
        distances: &mut Vec<f32>,
    ) where
        C: Clone,
    {
        let pop_len = ecosystem.population().len();

        for i in 0..pop_len {
            if let Some(species_id) = assignments[i] {
                ecosystem.add_species_member(species_id, i);
                continue;
            }

            let genotype = ecosystem.get_genotype(i).unwrap();
            let maybe_idx = ecosystem
                .species()
                .map(|specs| {
                    for (species_idx, species) in specs.iter().enumerate() {
                        let dist = self
                            .diversity
                            .distance(genotype, species.mascot().genotype());

                        distances.push(dist);

                        if dist < self.threashold {
                            return Some(species_idx);
                        }
                    }
                    None
                })
                .flatten();

            match maybe_idx {
                Some(idx) => ecosystem.add_species_member(idx, i),
                None => {
                    if let Some(pheno) = ecosystem.get_phenotype(i) {
                        let new_species = Species::new(generation, pheno);
                        ecosystem.push_species(new_species);
                    }
                }
            }
        }
    }

    #[inline]
    fn build_mascots(ecosystem: &Ecosystem<C>) -> Arc<Vec<Genotype<C>>>
    where
        C: Clone,
    {
        Arc::new(
            ecosystem
                .species_mascots()
                .into_iter()
                .map(|spec| spec.genotype().clone())
                .collect::<Vec<Genotype<C>>>(),
        )
    }

    #[inline]
    fn process_chunk(
        population: Arc<RwLock<Vec<(usize, Genotype<C>)>>>,
        species_mascots: Arc<Vec<Genotype<C>>>,
        threshold: f32,
        diversity: Arc<dyn Distance<Genotype<C>>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
    ) {
        let mut inner_distances = Vec::new();
        for (idx, individual) in population.read().unwrap().iter() {
            let mut assigned = None;
            for (idx, sp) in species_mascots.iter().enumerate() {
                let dist = diversity.distance(&individual, &sp);
                inner_distances.push(dist);

                if dist < threshold {
                    assigned = Some(idx);
                    break;
                }
            }

            assignments.lock().unwrap()[*idx] = assigned;
        }

        distances.lock().unwrap().extend(inner_distances);
    }
}

impl<C> EngineStep<C> for SpeciateStep<C>
where
    C: Chromosome + PartialEq + Clone + 'static,
{
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) -> Result<()> {
        ecosystem.generate_mascots();

        let pop_len = ecosystem.population().len();
        if pop_len == 0 {
            return Ok(());
        }

        let mascots = Self::build_mascots(ecosystem);
        let distance_capacity = pop_len * mascots.len().max(1);
        let distances = Arc::new(Mutex::new(Vec::with_capacity(distance_capacity)));
        let assignments = Arc::new(Mutex::new(vec![None; pop_len]));

        self.build_chunks_and_batches(
            ecosystem,
            Arc::clone(&mascots),
            Arc::clone(&assignments),
            Arc::clone(&distances),
        )?;

        let mut distances_guard = distances.lock().unwrap();
        let assignments_guard = assignments.lock().unwrap();
        self.assign_unassigned(
            generation,
            ecosystem,
            &assignments_guard,
            &mut distances_guard,
        );

        let removed_species = ecosystem.remvove_dead_species();

        metrics.upsert(metric_names::SPECIES_DISTANCE_DIST, &*distances_guard);
        metrics.upsert(metric_names::SPECIES_DIED, removed_species);

        ecosystem.fitness_share(&self.objective);

        Ok(())
    }
}
