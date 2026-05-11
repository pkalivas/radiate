use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Executor, MetricSet, Objective, Phenotype, Population, Rate, Species,
    diversity::Diversity, metric_names, random_provider,
};
use radiate_error::Result;
use std::sync::{Arc, Mutex, RwLock};

pub struct SpeciateStep<C>
where
    C: Chromosome,
{
    pub(crate) threshold: Rate,
    pub(crate) objective: Objective,
    pub(crate) distance: Arc<dyn Diversity<C>>,
    pub(crate) executor: Arc<Executor>,
    pub(crate) distances: Arc<Mutex<Vec<f32>>>,
    pub(crate) assignments: Arc<Mutex<Vec<Option<(usize, f32)>>>>,
}

impl<C: Chromosome> SpeciateStep<C> {
    pub fn new(
        threshold: Rate,
        objective: Objective,
        distance: Arc<dyn Diversity<C>>,
        executor: Arc<Executor>,
    ) -> Self {
        Self {
            threshold,
            objective,
            distance,
            executor,
            distances: Arc::new(Mutex::new(Vec::new())),
            assignments: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<C> SpeciateStep<C>
where
    C: Chromosome + 'static,
{
    fn assign_species(
        &self,
        generation: usize,
        threshold: f32,
        ecosystem: &mut Ecosystem<C>,
        mascots: Arc<Vec<Phenotype<C>>>,
        assignments: Arc<Mutex<Vec<Option<(usize, f32)>>>>,
    ) -> Result<()>
    where
        C: Clone,
    {
        let pop_len = ecosystem.population().len();
        let num_threads = self.executor.num_workers().max(1);
        let chunk_size = (pop_len as f32 / num_threads as f32).ceil() as usize;

        let mut batches = Vec::new();

        let mut empty_population = Population::empty();
        std::mem::swap(ecosystem.population_mut(), &mut empty_population);
        let population = Arc::new(RwLock::new(empty_population));

        for chunk_start in (0..pop_len).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(pop_len);

            let distance = Arc::clone(&self.distance);
            let assignments = Arc::clone(&assignments);
            let population = Arc::clone(&population);
            let species_snapshot = Arc::clone(&mascots);

            batches.push(move || {
                Self::process_chunk(
                    population,
                    species_snapshot,
                    threshold,
                    distance,
                    assignments,
                    chunk_start..chunk_end,
                );
            });
        }

        self.executor.submit_blocking(batches);

        std::mem::swap(ecosystem.population_mut(), &mut population.write().unwrap());
        self.assign_unassigned(
            generation,
            threshold,
            ecosystem,
            &assignments.lock().unwrap(),
        );

        Ok(())
    }

    #[inline]
    fn assign_unassigned(
        &self,
        generation: usize,
        threshold: f32,
        ecosystem: &mut Ecosystem<C>,
        assignments: &[Option<(usize, f32)>],
    ) where
        C: Clone,
    {
        let pop_len = ecosystem.population().len();
        let mut new_count = 0;
        let mut distances = self.distances.lock().unwrap();

        for (i, assignment) in assignments.iter().enumerate().take(pop_len) {
            if let Some((species_id, dist)) = assignment {
                ecosystem.add_species_member(*species_id, i);
                distances[i] = *dist;
                continue;
            }

            let mut best_dist = f32::MAX;
            let phenotype = ecosystem.get_phenotype(i).unwrap();
            let maybe_idx = ecosystem.species().and_then(|specs| {
                for (species_idx, species) in specs.iter().enumerate() {
                    let dist = self.distance.measure(phenotype, species.mascot());

                    best_dist = best_dist.min(dist);

                    if dist < threshold {
                        distances[i] = dist;
                        return Some(species_idx);
                    }
                }

                None
            });

            match maybe_idx {
                Some(idx) => ecosystem.add_species_member(idx, i),
                None => {
                    if let Some(pheno) = ecosystem.get_phenotype_mut(i) {
                        let new_species = Species::new(generation, pheno.clone());
                        let species_idx = ecosystem.push_species(new_species);

                        ecosystem.add_species_member(species_idx, i);
                        distances[i] = best_dist;

                        new_count += 1;
                    }
                }
            }
        }

        if new_count == pop_len {
            ecosystem.clear_species();
            distances.clear();
            distances.push(threshold);
            let idx = random_provider::range(0..pop_len);
            if let Some(phenotype) = ecosystem.get_phenotype(idx) {
                let new_species = Species::new(generation, (*phenotype).clone());
                let new_species_idx = ecosystem.push_species(new_species);

                for i in 0..pop_len {
                    if i != idx {
                        ecosystem.add_species_member(new_species_idx, i);
                    }
                }
            }
        }
    }

    #[inline]
    fn process_chunk(
        population: Arc<RwLock<Population<C>>>,
        species_mascots: Arc<Vec<Phenotype<C>>>,
        threshold: f32,
        distance: Arc<dyn Diversity<C>>,
        assignments: Arc<Mutex<Vec<Option<(usize, f32)>>>>,
        range: std::ops::Range<usize>,
    ) {
        let mut inner_assignments = Vec::new();

        let start = range.start;
        let reader = population.read().unwrap();
        for (idx, individual) in reader[range].iter().enumerate() {
            let mut assigned = None;
            for (spec_idx, sp) in species_mascots.iter().enumerate() {
                let dist = distance.measure(individual, sp);

                if dist < threshold {
                    assigned = Some((spec_idx, dist));
                    break;
                }
            }

            if assigned.is_some() {
                inner_assignments.push((start + idx, assigned));
            }
        }

        {
            let mut assignments = assignments.lock().unwrap();
            for (idx, assigned) in inner_assignments {
                assignments[idx] = assigned;
            }
        }
    }

    #[inline]
    fn generate_mascots(ecosystem: &mut Ecosystem<C>) -> Arc<Vec<Phenotype<C>>>
    where
        C: Clone,
    {
        ecosystem.generate_mascots();

        Arc::new(
            ecosystem
                .species_mascots()
                .into_iter()
                .cloned()
                .collect::<Vec<Phenotype<C>>>(),
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

        let mut threshold = self.threshold.get(generation, metrics);
        // if threshold <= 0.0 {
        //     threshold = 0.1;
        // }

        let mascots = Self::generate_mascots(ecosystem);

        let distances = {
            let mut distances_guard = self.distances.lock().unwrap();
            distances_guard.clear();
            distances_guard.resize(pop_len, 0.0);
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
            threshold,
            ecosystem,
            Arc::clone(&mascots),
            Arc::clone(&assignments),
        )?;

        let rm_species_count = ecosystem.remove_dead_species();

        let distances = distances.lock().unwrap();
        metrics.upsert((metric_names::SPECIES_DISTANCE_DIST, &*distances));
        metrics.upsert((metric_names::SPECIES_DIED, rm_species_count));
        metrics.upsert((metric_names::SPECIES_THRESHOLD, threshold));

        ecosystem.fitness_share(&self.objective);

        Ok(())
    }
}
