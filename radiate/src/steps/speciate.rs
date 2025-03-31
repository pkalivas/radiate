use super::EngineStep;
use crate::{
    Chromosome, DiversityMeasure, GeneticEngineParams, Metric, Objective, Phenotype, Population,
    Rate, Score, Species, metric_names, random_provider,
    thread_pool::{ThreadPool, WaitGroup},
};
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

pub struct SpeciateStep<C: Chromosome> {
    objective: Objective,
    diversity: Arc<dyn DiversityMeasure<C>>,
    rate: Rate,
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
            .map(|score| score.inner().clone().unwrap() / species.len() as f32)
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

            let threshold = self.rate.get();
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

                    if dist < self.rate.get() {
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
                thread_pool: Arc::clone(&params.thread_pool()),
                rate: Rate::Static(Cell::new(params.species_threshold())),
            }));
        }
        None
    }
}

// let previous_best = if species.is_empty() {
//     None
// } else {
//     species
//         .iter()
//         .map(|s| s.stagnation_tracker().current_score())
//         .max_by(|a, b| {
//             if let Objective::Single(Optimize::Maximize) = &self.objective {
//                 a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
//             } else {
//                 b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
//             }
//         })
//         .cloned()
// };

// let current_best = population.get(0).score();

// println!("THRESHOLD: {}", self.rate.get());

// let mean_distance = if distances.is_empty() {
//     0.0
// } else {
//     distances.iter().sum::<f32>() / distances.len() as f32
// };

// if let (Some(previous), Some(current)) = (previous_best, current_best) {
//     let prev = previous.as_f32();
//     let curr = current.as_f32();

//     let scaled_improvement = if prev > 0.0 {
//         if let Objective::Single(Optimize::Minimize) = &self.objective {
//             (prev - curr) / prev
//         } else {
//             (curr - prev) / prev
//         }
//     } else {
//         curr
//     };

//     println!(
//         "Previous Best: {:.4}, Current Best: {:.4}, Improvement: {:.4}",
//         prev, curr, scaled_improvement
//     );

//     self.rate.update((1.0 - scaled_improvement) * mean_distance);
// }

// rate: Rate::Diversity {
//     current: Cell::new(params.species_threshold()),
//     alpha: 0.0001,
//     min: 0.1,
//     max: 100.0,
// },
