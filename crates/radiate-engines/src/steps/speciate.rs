use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Executor, Genotype, MetricSet, Objective, Score, Species,
    diversity::Distance, metric_names, random_provider,
};
use radiate_error::Result;
use std::sync::{Arc, Mutex, RwLock};

pub struct SpeciateStep<C>
where
    C: Chromosome,
{
    pub(crate) threashold: f32,
    pub(crate) objective: Objective,
    pub(crate) distance: Arc<dyn Distance<Genotype<C>>>,
    pub(crate) executor: Arc<Executor>,
    pub(crate) distances: Arc<Mutex<Vec<f32>>>,
    pub(crate) assignments: Arc<Mutex<Vec<Option<usize>>>>,
}

impl<C> SpeciateStep<C>
where
    C: Chromosome + 'static,
{
    fn assign_species(
        &self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        mascots: Arc<Vec<Genotype<C>>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
    ) -> Result<()>
    where
        C: Clone,
    {
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
                    .take(chunk_end - chunk_start)
                    .try_fold(Vec::new(), |mut acc, (idx, pheno)| {
                        let genotype = pheno.take_genotype()?;
                        acc.push((idx, genotype));
                        Ok::<_, radiate_core::RadiateError>(acc)
                    })?,
            ));

            let threshold = self.threashold;
            let distance = Arc::clone(&self.distance);
            let assignments = Arc::clone(&assignments);
            let distances = Arc::clone(&distances);
            let population = Arc::clone(&chunk_population);
            let species_snapshot = Arc::clone(&mascots);

            batches.push(move || {
                Self::process_chunk(
                    population,
                    species_snapshot,
                    threshold,
                    distance,
                    assignments,
                    distances,
                );
            });

            chunked_members.push(chunk_population);
        }

        self.executor.submit_blocking(batches);

        self.restore_genotypes(ecosystem, chunked_members);
        self.assign_unassigned(generation, ecosystem, &assignments.lock().unwrap());

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
                        if species.age(generation) != 0 {
                            continue;
                        }

                        let dist = self
                            .distance
                            .distance(genotype, species.mascot().genotype());

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
    fn fitness_share(&self, ecosystem: &mut Ecosystem<C>)
    where
        C: PartialEq,
    {
        if let Some(species) = ecosystem.species_mut() {
            let mut scores = Vec::with_capacity(species.len());
            for spec in species.iter() {
                let adjusted = Self::adjust_scores(spec).iter().sum::<Score>();

                scores.push(adjusted);
            }

            let total_score = scores.iter().sum::<Score>();
            for (i, spec) in species.iter_mut().enumerate() {
                let spec_score = scores[i].clone();
                let adjusted_score = spec_score / total_score.clone();
                spec.update_score(adjusted_score, &self.objective);
            }

            self.objective.sort(species);
        }
    }

    #[inline]
    fn adjust_scores(species: &Species<C>) -> Vec<Score> {
        species
            .population
            .get_scores()
            .map(|score| (*score).clone() / species.len() as f32)
            .collect()
    }

    #[inline]
    fn process_chunk(
        population: Arc<RwLock<Vec<(usize, Genotype<C>)>>>,
        species_mascots: Arc<Vec<Genotype<C>>>,
        threshold: f32,
        distance: Arc<dyn Distance<Genotype<C>>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
    ) {
        let mut inner_distances = Vec::new();
        let mut inner_assignments = Vec::new();

        for (idx, individual) in population.read().unwrap().iter() {
            let mut assigned = None;
            for (idx, sp) in species_mascots.iter().enumerate() {
                let dist = distance.distance(&individual, &sp);
                inner_distances.push(dist);

                if dist < threshold {
                    assigned = Some(idx);
                    break;
                }
            }

            if assigned.is_some() {
                inner_assignments.push((*idx, assigned));
            }
        }

        {
            let mut assignments = assignments.lock().unwrap();
            for (idx, assigned) in inner_assignments {
                assignments[idx] = assigned;
            }
        }

        {
            distances.lock().unwrap().extend(inner_distances);
        }
    }

    #[inline]
    fn generate_mascots(ecosystem: &mut Ecosystem<C>) -> Arc<Vec<Genotype<C>>>
    where
        C: Clone,
    {
        // Update mascots for each species by selecting a random member from the species population
        // to be the new mascot for the next generation. This follows the NEAT algorithm approach.
        if let Some(species) = ecosystem.species_mut() {
            for spec in species {
                let idx = random_provider::range(0..spec.population.len());
                spec.population().get(idx).cloned().map(|phenotype| {
                    spec.set_new_mascot(phenotype);
                });
            }
        }

        Arc::new(
            ecosystem
                .species_mascots()
                .into_iter()
                .map(|spec| spec.genotype().clone())
                .collect::<Vec<Genotype<C>>>(),
        )
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
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        let pop_len = ecosystem.population().len();
        if pop_len == 0 {
            return Ok(());
        }

        let mascots = Self::generate_mascots(ecosystem);

        let distances = {
            let distance_capacity = pop_len * mascots.len().max(1);
            let mut distances_guard = self.distances.lock().unwrap();
            distances_guard.clear();
            distances_guard.reserve_exact(distance_capacity);
            Arc::clone(&self.distances)
        };

        let assignments = {
            let mut assignments_guard = self.assignments.lock().unwrap();
            assignments_guard.clear();
            assignments_guard.resize(pop_len, None);
            Arc::clone(&self.assignments)
        };

        self.assign_species(
            generation,
            ecosystem,
            Arc::clone(&mascots),
            Arc::clone(&assignments),
            Arc::clone(&distances),
        )?;

        let rm_species_count = ecosystem.remove_dead_species();

        metrics.upsert((metric_names::SPECIES_DIED, rm_species_count));

        self.fitness_share(ecosystem);

        Ok(())
    }
}
