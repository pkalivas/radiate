use super::EngineStep;
use crate::{
    Chromosome, DiversityMeasure, GeneticEngineParams, Metric, Objective, Phenotype, Population,
    Score, Species, metric_names, random_provider,
    thread_pool::{ThreadPool, WaitGroup},
};
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

pub struct SpeciateStep<C: Chromosome> {
    objective: Objective,
    diversity: Arc<dyn DiversityMeasure<C>>,
    threshold: Cell<f32>,
    thread_pool: Arc<ThreadPool>,
}

impl<C: Chromosome> SpeciateStep<C> {
    pub fn generate_mascots(&self, species: &mut Vec<Species<C>>) {
        for species in species.iter_mut() {
            let random_phenotype = random_provider::choose(species.population().as_ref());
            species.set_mascot(random_phenotype.clone());
            species.population_mut().clear();
        }
    }

    pub fn fitness_share(&self, species: &mut Vec<Species<C>>) {
        let mut total_species_score = Score::default();
        for species in species.iter() {
            total_species_score =
                total_species_score + Self::adjust_scores(species).iter().sum::<Score>();
        }

        for species in species.iter_mut() {
            let adjusted_score =
                Self::adjust_scores(species).iter().sum::<Score>() / total_species_score.clone();
            species.update_score(adjusted_score, &self.objective);
        }

        self.objective.sort(species);
    }

    fn adjust_scores(species: &Species<C>) -> Vec<Score> {
        species
            .population()
            .get_scores()
            .iter()
            .map(|score| score.clone() / species.len() as f32)
            .collect()
    }

    fn process_chunk(
        chunk_start: usize,
        population_chunk: Vec<Phenotype<C>>,
        species_snapshot: Vec<Species<C>>,
        threshold: f32,
        diversity: Arc<dyn DiversityMeasure<C>>,
        assignments: Arc<Mutex<Vec<Option<usize>>>>,
        distances: Arc<Mutex<Vec<f32>>>,
    ) {
        let mut inner_distances = Vec::new();
        for (i, individual) in population_chunk.iter().enumerate() {
            let mut assigned = None;
            for (idx, sp) in species_snapshot.iter().enumerate() {
                let dist = diversity.diversity(individual, &sp.mascot());
                inner_distances.push(dist);

                if dist < threshold {
                    assigned = Some(idx);
                    break;
                }
            }

            assignments.lock().unwrap()[chunk_start + i] = assigned;
        }

        distances.lock().unwrap().extend(inner_distances);
    }
}

impl<C, T> EngineStep<C, T> for SpeciateStep<C>
where
    C: Chromosome + 'static,
    T: Clone + Send,
{
    fn execute(
        &self,
        generation: usize,
        population: &mut Population<C>,
        species: &mut Vec<Species<C>>,
    ) -> Vec<Metric> {
        self.generate_mascots(species);

        let wg = WaitGroup::new();
        let num_threads = self.thread_pool.num_workers();
        let pop_len = population.len();
        let chunk_size = (pop_len as f32 / num_threads as f32).ceil() as usize;

        let distances = Arc::new(Mutex::new(Vec::with_capacity(pop_len * species.len())));
        let assignments = Arc::new(Mutex::new(vec![None; pop_len]));

        for chunk_start in (0..pop_len).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(pop_len);
            let population_snapshot = population[chunk_start..chunk_end]
                .iter()
                .map(|pheno| Phenotype::clone(pheno))
                .collect::<Vec<Phenotype<C>>>();
            let species_snapshot = species.iter().cloned().collect::<Vec<Species<C>>>();

            let threshold = self.threshold.get();
            let diversity = Arc::clone(&self.diversity);
            let assignments = Arc::clone(&assignments);
            let distances = Arc::clone(&distances);

            self.thread_pool.group_submit(&wg, move || {
                Self::process_chunk(
                    chunk_start,
                    population_snapshot,
                    species_snapshot,
                    threshold,
                    diversity,
                    assignments,
                    distances,
                );
            });
        }

        wg.wait();

        let assignments = assignments.lock().unwrap();
        let mut distances = distances.lock().unwrap();
        for i in 0..population.len() {
            if let Some(species_id) = assignments[i] {
                species[species_id].add_member(population.get(i));
            } else {
                let mut found = false;
                for species in species.iter_mut() {
                    let dist = self
                        .diversity
                        .diversity(population.get(i), &species.mascot());

                    distances.push(dist);

                    if dist < self.threshold.get() {
                        species.add_member(population.get(i));
                        found = true;
                        break;
                    }
                }

                if !found {
                    let phenotype = population.get(i);
                    let new_species = Species::new(Phenotype::clone(phenotype), generation);

                    species.push(new_species);
                }
            }
        }

        let before_species = species.len();
        species.retain(|s| s.len() > 0);

        self.fitness_share(species);

        vec![
            Metric::new_distribution(metric_names::SPECIES_DISTANCE_DIST)
                .with_distribution(&distances),
            Metric::new_value(metric_names::SPECIES_DIED)
                .with_value((before_species - species.len()) as f32),
        ]
    }

    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        if let Some(distance) = params.distance() {
            return Some(Box::new(SpeciateStep {
                objective: params.objective().clone(),
                diversity: Arc::clone(&distance),
                threshold: Cell::new(params.species_threshold()),
                thread_pool: Arc::clone(&params.thread_pool()),
            }));
        }
        None
    }
}
