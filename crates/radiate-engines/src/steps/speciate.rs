use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Executor, MetricSet, Objective, Phenotype, Population, Score, Species,
    diversity::Diversity, metric_names, random_provider,
};
use radiate_error::Result;
use std::sync::{Arc, Mutex, RwLock};

pub struct SpeciateStep<C>
where
    C: Chromosome,
{
    pub(crate) threshold: f32,
    pub(crate) objective: Objective,
    pub(crate) distance: Arc<dyn Diversity<C>>,
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
        mascots: Arc<Vec<Phenotype<C>>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
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

            let threshold = self.threshold;
            let distance = Arc::clone(&self.distance);
            let assignments = Arc::clone(&assignments);
            let distances = Arc::clone(&distances);
            let population = Arc::clone(&population);
            let species_snapshot = Arc::clone(&mascots);

            batches.push(move || {
                Self::process_chunk(
                    population,
                    species_snapshot,
                    threshold,
                    distance,
                    assignments,
                    distances,
                    chunk_start..chunk_end,
                );
            });
        }

        self.executor.submit_blocking(batches);

        std::mem::swap(ecosystem.population_mut(), &mut population.write().unwrap());
        self.assign_unassigned(generation, ecosystem, &assignments.lock().unwrap());

        Ok(())
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

            let phenotype = ecosystem.get_phenotype(i).unwrap();
            let maybe_idx = ecosystem
                .species()
                .map(|specs| {
                    for (species_idx, species) in specs.iter().enumerate() {
                        if species.age(generation) != 0 {
                            continue;
                        }

                        let dist = self.distance.measure(phenotype, species.mascot());

                        if dist < self.threshold {
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
        population: Arc<RwLock<Population<C>>>,
        species_mascots: Arc<Vec<Phenotype<C>>>,
        threshold: f32,
        distance: Arc<dyn Diversity<C>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
        range: std::ops::Range<usize>,
    ) {
        let mut inner_distances = Vec::new();
        let mut inner_assignments = Vec::new();

        let start = range.start;
        let reader = population.read().unwrap();
        for (idx, individual) in reader[range].iter().enumerate() {
            let mut assigned = None;
            for (idx, sp) in species_mascots.iter().enumerate() {
                let dist = distance.measure(individual.borrow(), &sp);
                inner_distances.push(dist);

                if dist < threshold {
                    assigned = Some(idx);
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

        {
            distances.lock().unwrap().extend(inner_distances);
        }
    }

    #[inline]
    fn generate_mascots(ecosystem: &mut Ecosystem<C>) -> Arc<Vec<Phenotype<C>>>
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
                .map(|spec| spec.clone())
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
