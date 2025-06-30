// use std::{
//     collections::VecDeque,
//     sync::{Arc, RwLock},
// };

// use std::any::Any;

// /// Trait for extracting behavioral descriptors from phenotypes.
// /// Behavioral descriptors characterize what an individual does, not how well it does it.
// pub trait BehavioralDescriptor<T>: Send + Sync {
//     type Descriptor: Send + Sync;

//     /// Extract a behavioral descriptor from a phenotype
//     fn extract_descriptor(&self, phenotype: &T) -> Self::Descriptor;

//     /// Calculate distance between two behavioral descriptors
//     fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32;
// }

// /// Default behavioral descriptor that uses the phenotype itself
// pub struct PhenotypeDescriptor;

// impl<T: Clone + PartialEq + Send + Sync> BehavioralDescriptor<T> for PhenotypeDescriptor {
//     type Descriptor = T;

//     fn extract_descriptor(&self, phenotype: &T) -> Self::Descriptor {
//         phenotype.clone()
//     }

//     fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
//         if a == b { 0.0 } else { 1.0 }
//     }
// }

// /// Behavioral descriptor for numeric vectors (common case)
// pub struct VectorDescriptor;

// impl BehavioralDescriptor<Vec<f32>> for VectorDescriptor {
//     type Descriptor = Vec<f32>;

//     fn extract_descriptor(&self, phenotype: &Vec<f32>) -> Self::Descriptor {
//         phenotype.clone()
//     }

//     fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
//         if a.len() != b.len() {
//             return f32::INFINITY;
//         }

//         let sum_squared_diff: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();

//         sum_squared_diff.sqrt()
//     }
// }

// /// Behavioral descriptor for function outputs
// pub struct FunctionOutputDescriptor<F, T, D>
// where
//     F: Fn(&T) -> D + Send + Sync,
//     D: Send + Sync,
// {
//     test_inputs: Vec<T>,
//     output_fn: F,
//     _phantom: std::marker::PhantomData<D>,
// }

// impl<F, T, D> FunctionOutputDescriptor<F, T, D>
// where
//     F: Fn(&T) -> D + Send + Sync,
//     D: Clone + PartialEq + Send + Sync,
// {
//     pub fn new(test_inputs: Vec<T>, output_fn: F) -> Self {
//         Self {
//             test_inputs,
//             output_fn,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

// impl<F, T, D> BehavioralDescriptor<T> for FunctionOutputDescriptor<F, T, D>
// where
//     F: Fn(&T) -> D + Send + Sync,
//     T: Send + Sync,
//     D: Send + Sync,
// {
//     type Descriptor = Vec<D>;

//     fn extract_descriptor(&self, phenotype: &T) -> Self::Descriptor {
//         self.test_inputs
//             .iter()
//             .map(|input| (self.output_fn)(input))
//             .collect()
//     }

//     fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
//         if a.len() != b.len() {
//             return f32::INFINITY;
//         }

//         let sum_squared_diff: f32 = a
//             .iter()
//             .zip(b.iter())
//             .map(|(x, y)| {
//                 if let (Some(x_val), Some(y_val)) = (self.to_f32(x), self.to_f32(y)) {
//                     (x_val - y_val).powi(2)
//                 } else {
//                     1.0 // Default distance for non-numeric values
//                 }
//             })
//             .sum();

//         sum_squared_diff.sqrt()
//     }
// }

// impl<F, T, D> FunctionOutputDescriptor<F, T, D>
// where
//     F: Fn(&T) -> D + Send + Sync,
//     T: Send + Sync,
//     D: Send + Sync,
// {
//     fn to_f32(&self, value: &D) -> Option<f32> {
//         // Try to convert to f32 - this is a simplified version
//         // In practice, you'd want to implement this based on the actual type D
//         None
//     }
// }
