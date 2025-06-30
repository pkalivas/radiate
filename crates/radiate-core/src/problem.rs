use super::{Chromosome, Codec, Genotype, Score};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

pub trait FitnessFn<T> {
    fn eval(&self, phenotype: &T) -> impl Into<Score>;
}

pub struct NoveltyObjective<T> {
    descriptor_fn: Arc<dyn Fn(&T) -> Vec<f32> + Send + Sync>,
    distance_fn: Arc<dyn Fn(&[f32], &[f32]) -> f32 + Send + Sync>,
    archive: Arc<RwLock<VecDeque<Vec<f32>>>>,
    k: usize,
}

//
//
/// [Problem] represents the interface for the fitness function or evaluation and encoding/decoding
/// of a genotype to a phenotype within the genetic algorithm framework.
///
/// To run the genetic algorithm the three things that must be supplied are the encoding & decoding of
/// the [Genotype] and the fitness function. [Problem] wraps all three into a
/// single trait that can be supplied to the engine builder.
pub trait Problem<C: Chromosome, T>: Send + Sync {
    fn encode(&self) -> Genotype<C>;
    fn decode(&self, genotype: &Genotype<C>) -> T;
    fn eval(&self, individual: &Genotype<C>) -> Score;
}

/// [EngineProblem] is a generic, base level concrete implementation of the [Problem] trait that is the
/// default problem used by the engine if none other is specified during building. We take the
/// [Codec] and the fitness function from the user and simply wrap them into this struct.
pub struct EngineProblem<C, T>
where
    C: Chromosome,
{
    pub codec: Arc<dyn Codec<C, T>>,
    pub fitness_fn: Arc<dyn Fn(T) -> Score + Send + Sync>,
}

impl<C: Chromosome, T> Problem<C, T> for EngineProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        let phenotype = self.decode(individual);
        (self.fitness_fn)(phenotype)
    }
}

unsafe impl<C: Chromosome, T> Send for EngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for EngineProblem<C, T> {}

// pub struct NoveltyProblem<C, T>
// where
//     C: Chromosome,
// {
//     pub codec: Arc<dyn Codec<C, T>>,
//     pub fitness_fn: Arc<dyn Fn(T) -> Score + Send + Sync>,
//     pub novelty: NoveltyObjective<T>,
// }

// impl<T> NoveltyObjective<T> {
//     pub fn new<F, G>(descriptor_fn: F, distance_fn: G, k: usize) -> Self
//     where
//         F: Fn(&T) -> Vec<f32> + Send + Sync + 'static,
//         G: Fn(&[f32], &[f32]) -> f32 + Send + Sync + 'static,
//     {
//         NoveltyObjective {
//             descriptor_fn: Arc::new(descriptor_fn),
//             distance_fn: Arc::new(distance_fn),
//             archive: Arc::new(RwLock::new(VecDeque::new())),
//             k,
//         }
//     }
// }

// impl<T> FitnessFn<T> for NoveltyObjective<T> {
//     fn eval(&self, phenotype: &T) -> impl Into<Score> {
//         let descriptor = (self.descriptor_fn)(phenotype);

//         let archive = self.archive.read().unwrap();
//         if archive.is_empty() {
//             return 1.0; // base novelty
//         }

//         let mut distances = archive
//             .iter()
//             .map(|past| (self.distance_fn)(&descriptor, past))
//             .collect::<Vec<f32>>();

//         distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

//         let k = self.k.min(distances.len());
//         let novelty = distances.iter().take(k).sum::<f32>() / (k as f32);

//         drop(archive);

//         let mut writer = self.archive.write().unwrap();

//         if writer.len() > 1000 {
//             writer.pop_front();
//         }

//         writer.push_back(descriptor);

//         novelty
//     }
// }

// impl<C: Chromosome> Problem<C, f32> for NoveltyObjective<C> {
//     fn eval(&self, individual: &C) -> f32 {
//         let descriptor = (self.descriptor_fn)(individual);

//         let archive = self.archive.read().unwrap();
//         if archive.is_empty() {
//             return 1.0; // base novelty
//         }

//         let mut distances: Vec<f32> = archive
//             .iter()
//             .map(|past| (self.distance_fn)(&descriptor, past))
//             .collect();

//         distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
//         let k = self.k.min(distances.len());
//         let novelty = distances.iter().take(k).sum::<f32>() / (k as f32);
//         drop(archive);

//         self.archive.write().unwrap().push(descriptor);

//         novelty
//     }
// }

// pub struct NoveltyObjective<C> {
//     descriptor_fn: Arc<dyn Fn(&C) -> Vec<f32> + Send + Sync>,
//     distance_fn: Arc<dyn Fn(&[f32], &[f32]) -> f32 + Send + Sync>,
//     archive: RwLock<Vec<Vec<f32>>>,
//     k: usize,
// }

// pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
//     let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
//     let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
//     let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
//     1.0 - dot / (norm_a * norm_b + 1e-8)
// }
