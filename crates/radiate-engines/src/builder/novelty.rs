use radiate_core::{Chromosome, Epoch, Genotype};
use std::sync::Arc;

use crate::GeneticEngineBuilder;

// #[derive(Clone)]
// pub struct NoveltyParams<C: Chromosome> {
//     pub descriptor_fn: Option<Arc<dyn Fn(&Genotype<C>) -> Vec<f32> + Send + Sync>>,
//     pub distance_fn: Option<Arc<dyn Fn(&[f32], &[f32]) -> f32 + Send + Sync>>,
//     pub archive_size: usize,
//     pub k_nearest: usize,
//     pub novelty_threshold: f32,
//     pub novelty_weight: f32,
//     pub fitness_weight: f32,
// }

// impl<C, T, E> GeneticEngineBuilder<C, T, E>
// where
//     C: Chromosome + PartialEq + Clone,
//     T: Clone + Send,
//     E: Epoch<C>,
// {
//     /// Set the novelty parameters of the genetic engine. This is useful if you want to provide a custom novelty search configuration.
//     /// If this is not set, the genetic engine will not use novelty search.
//     pub fn novelty_descriptor<F>(mut self, descriptor_fn: F) -> Self
//     where
//         F: Fn(&Genotype<C>) -> Vec<f32> + 'static + Send + Sync,
//     {
//         self.params.novelty_params.descriptor_fn = Some(Arc::new(descriptor_fn));
//         self
//     }

//     pub fn novelty_distance<F>(mut self, distance_fn: F) -> Self
//     where
//         F: Fn(&[f32], &[f32]) -> f32 + 'static + Send + Sync,
//     {
//         self.params.novelty_params.distance_fn = Some(Arc::new(distance_fn));
//         self
//     }

//     pub fn novelty_k_nearest(mut self, k: usize) -> Self {
//         self.params.novelty_params.k_nearest = k;
//         self
//     }

//     pub fn novelty_archive_size(mut self, size: usize) -> Self {
//         self.params.novelty_params.archive_size = size;
//         self
//     }

//     pub fn novelty_threshold(mut self, threshold: f32) -> Self {
//         self.params.novelty_params.novelty_threshold = threshold;
//         self
//     }

//     pub fn novelty_weights(mut self, novelty_weight: f32, fitness_weight: f32) -> Self {
//         self.params.novelty_params.novelty_weight = novelty_weight;
//         self.params.novelty_params.fitness_weight = fitness_weight;
//         self
//     }
// }
