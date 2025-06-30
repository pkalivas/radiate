// use std::sync::{Arc, RwLock};

// use radiate_core::{
//     Chromosome, Ecosystem, EngineStep, Executor, Genotype, MetricSet, Objective, Optimize, Score,
// };

// pub struct NoveltyStep<C: Chromosome> {
//     pub(crate) objective: Objective,
//     pub(crate) executor: Arc<Executor>,
//     pub(crate) descriptor: Arc<dyn Fn(&Genotype<C>) -> Vec<f32> + Send + Sync>,
//     pub(crate) distance: Arc<dyn Fn(&[f32], &[f32]) -> f32 + Send + Sync>,
//     pub(crate) archive: Arc<RwLock<Vec<Vec<f32>>>>,
//     pub(crate) k_nearest: usize,
//     pub(crate) threshold: f32,
//     pub(crate) weights: (f32, f32),
// }

// impl<C> NoveltyStep<C>
// where
//     C: Chromosome + PartialEq + 'static,
// {
//     fn normalized_novelty_score(&self, descriptor: &Vec<f32>) -> f32 {
//         let archive = self.archive.read().unwrap();
//         if archive.is_empty() {
//             return 1.0;
//         }

//         let mut distances: Vec<f32> = archive
//             .iter()
//             .map(|archived| (self.distance)(descriptor, archived))
//             .collect();

//         // Find min and max distances for normalization
//         let min_distance = distances.iter().fold(f32::INFINITY, |a, &b| a.min(b));
//         let max_distance = distances.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

//         if max_distance == min_distance {
//             return 0.0; // All distances are the same
//         }

//         // Use k-nearest average, but normalize it
//         distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
//         let k = std::cmp::min(self.k_nearest, distances.len());
//         let k_nearest_distances = &distances[..k];
//         let avg_distance = k_nearest_distances.iter().sum::<f32>() / k as f32;

//         // Normalize to [0, 1]
//         (avg_distance - min_distance) / (max_distance - min_distance)
//     }

//     fn update_archive(&self, descriptors: &Vec<Vec<f32>>) {
//         let mut archive = self.archive.write().unwrap();
//         for descriptor in descriptors {
//             let novelty_score = self.normalized_novelty_score(descriptor);
//             if novelty_score >= self.threshold {
//                 archive.push(descriptor.clone());
//             }
//         }
//     }

//     fn compute_novelty_scores(&self, ecosystem: &Ecosystem<C>) -> Vec<f32> {
//         ecosystem
//             .population
//             .iter()
//             .map(|ind| {
//                 let descriptor = (self.descriptor)(ind.genotype());
//                 self.normalized_novelty_score(&descriptor)
//             })
//             .collect()
//     }

//     fn adjust_fitness(&self, ecosystem: &mut Ecosystem<C>, novelty_scores: &Vec<f32>) {
//         for (ind, &novelty) in ecosystem.population.iter_mut().zip(novelty_scores.iter()) {
//             let adjusted = match self.objective {
//                 Objective::Single(_) => Score::from(
//                     ind.score().map(|score| score.as_f32()).unwrap_or(0.0) * self.weights.0
//                         + novelty * self.weights.1,
//                 ),
//                 Objective::Multi(_) => Score::from(
//                     ind.score()
//                         .map(|score| {
//                             score
//                                 .values
//                                 .iter()
//                                 .map(|v| v * self.weights.0 + novelty * self.weights.1)
//                                 .collect::<Vec<f32>>()
//                         })
//                         .unwrap(),
//                 ),
//             };

//             ind.set_score(Some(adjusted));
//         }
//     }
// }

// impl<C> EngineStep<C> for NoveltyStep<C>
// where
//     C: Chromosome + PartialEq + 'static,
// {
//     #[inline]
//     fn execute(&mut self, _: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {}
// }
