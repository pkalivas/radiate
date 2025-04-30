use radiate_core::{
    Chromosome, Diversity, Ecosystem, EngineStep, Genotype, MetricSet, Objective, Species,
    metric_names,
    thread_pool::{ThreadPool, WaitGroup},
};
use std::sync::{Arc, Mutex, RwLock};

pub struct SpeciateStep<C>
where
    C: Chromosome,
{
    pub(crate) threashold: f32,
    pub(crate) objective: Objective,
    pub(crate) diversity: Arc<dyn Diversity<C>>,
    pub(crate) thread_pool: Arc<ThreadPool>,
}

impl<C> SpeciateStep<C>
where
    C: Chromosome + 'static,
{
    fn process_chunk(
        chunk_range: std::ops::Range<usize>,
        population: Arc<RwLock<Vec<(usize, Genotype<C>)>>>,
        species_snapshot: Arc<Vec<Genotype<C>>>,
        threshold: f32,
        diversity: Arc<dyn Diversity<C>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
    ) {
        let mut inner_distances = Vec::new();
        for (i, individual) in population.read().unwrap().iter().enumerate() {
            let mut assigned = None;
            for (idx, sp) in species_snapshot.iter().enumerate() {
                let dist = diversity.measure(&individual.1, &sp);
                inner_distances.push(dist);

                if dist < threshold {
                    assigned = Some(idx);
                    break;
                }
            }

            assignments.lock().unwrap()[chunk_range.start + i] = assigned;
        }

        distances.lock().unwrap().extend(inner_distances);
    }
}

impl<C> EngineStep<C> for SpeciateStep<C>
where
    C: Chromosome + 'static,
{
    fn execute(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        ecosystem.generate_mascots();

        let wg = WaitGroup::new();
        let num_threads = self.thread_pool.num_workers();
        let pop_len = ecosystem.population().len();
        let chunk_size = (pop_len as f32 / num_threads as f32).ceil() as usize;
        let mut chunked_members = Vec::new();

        let species_snapshot = Arc::new(
            ecosystem
                .species_mascots()
                .into_iter()
                .map(|spec| spec.genotype().clone())
                .collect::<Vec<Genotype<C>>>(),
        );
        let distances = Arc::new(Mutex::new(Vec::with_capacity(
            pop_len * species_snapshot.len(),
        )));
        let assignments = Arc::new(Mutex::new(vec![None; pop_len]));

        for chunk_start in (0..pop_len).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(pop_len);
            let chunk_range = chunk_start..chunk_end;

            let chunk_population = Arc::new(RwLock::new(
                ecosystem
                    .population_mut()
                    .iter_mut()
                    .enumerate()
                    .skip(chunk_start)
                    .take(chunk_size)
                    .map(|(idx, pheno)| (idx, pheno.take_genotype()))
                    .collect::<Vec<_>>(),
            ));

            let threshold = self.threashold;
            let diversity = Arc::clone(&self.diversity);
            let assignments = Arc::clone(&assignments);
            let distances = Arc::clone(&distances);
            let population = Arc::clone(&chunk_population);
            let species_snapshot = Arc::clone(&species_snapshot);

            self.thread_pool.group_submit(&wg, move || {
                Self::process_chunk(
                    chunk_range,
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

        wg.wait();

        for chunks in chunked_members {
            let mut chunks = chunks.write().unwrap();
            let mut taken_genotypes = Vec::with_capacity(chunks.len());
            std::mem::swap(&mut *chunks, &mut taken_genotypes);

            for (idx, geno) in taken_genotypes {
                ecosystem.get_phenotype_mut(idx).unwrap().set_genotype(geno);
            }
        }

        let assignments = assignments.lock().unwrap();
        let mut distances = distances.lock().unwrap();
        for i in 0..ecosystem.population().len() {
            if let Some(species_id) = assignments[i] {
                ecosystem.add_species_member(species_id, i);
            } else {
                let genotype = ecosystem.get_genotype(i).unwrap();
                let maybe_idx = ecosystem
                    .species()
                    .map(|specs| {
                        for (species_idx, species) in specs.iter().enumerate() {
                            let dist = self
                                .diversity
                                .measure(genotype, &species.mascot().genotype());

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

        let before_species = ecosystem.species().as_ref().map_or(0, |s| s.len());
        ecosystem.species_mut().unwrap().retain(|s| s.len() > 0);
        let after_species = ecosystem.species().unwrap().len();

        metrics.upsert_distribution(metric_names::SPECIES_DISTANCE_DIST, &distances);
        metrics.upsert_value(
            metric_names::SPECIES_DIED,
            (before_species - after_species) as f32,
        );

        ecosystem.fitness_share(&self.objective);
    }
}
