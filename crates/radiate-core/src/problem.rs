use super::{Chromosome, Codec, Genotype, Score};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

pub trait FitnessFunction<T, S>: Send + Sync
where
    S: Into<Score>,
{
    fn evaluate(&self, individual: T) -> S;
}

impl<T, S, F> FitnessFunction<T, S> for F
where
    F: Fn(T) -> S + Send + Sync,
    S: Into<Score>,
{
    fn evaluate(&self, individual: T) -> S {
        (self)(individual)
    }
}

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

#[derive(Clone)]
pub struct NoveltySearch<T> {
    pub descriptor: Arc<dyn Fn(&T) -> Vec<f32> + Send + Sync>,
    pub distance: Arc<dyn Fn(&[f32], &[f32]) -> f32 + Send + Sync>,
    pub archive: Arc<RwLock<VecDeque<Vec<f32>>>>,
    pub k: usize,
    pub threshold: f32,
}

impl<T> NoveltySearch<T> {
    pub fn new<F, G>(descriptor_fn: F, distance_fn: G, k: usize, threshold: f32) -> Self
    where
        F: Fn(&T) -> Vec<f32> + Send + Sync + 'static,
        G: Fn(&[f32], &[f32]) -> f32 + Send + Sync + 'static,
    {
        NoveltySearch {
            descriptor: Arc::new(descriptor_fn),
            distance: Arc::new(distance_fn),
            archive: Arc::new(RwLock::new(VecDeque::new())),
            k,
            threshold,
        }
    }

    fn normalized_novelty_score(&self, descriptor: &Vec<f32>, archive: &VecDeque<Vec<f32>>) -> f32 {
        if archive.is_empty() {
            return 1.0;
        }

        let mut distances: Vec<f32> = archive
            .iter()
            .map(|archived| (self.distance)(descriptor, archived))
            .collect();

        let min_distance = distances.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_distance = distances.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        if max_distance == min_distance {
            return 0.0;
        }

        distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let k = std::cmp::min(self.k, distances.len());
        let k_nearest_distances = &distances[..k];
        let avg_distance = k_nearest_distances.iter().sum::<f32>() / k as f32;

        (avg_distance - min_distance) / (max_distance - min_distance)
    }
}

impl<T> FitnessFunction<T, Vec<f32>> for NoveltySearch<T> {
    fn evaluate(&self, individual: T) -> Vec<f32> {
        let descriptor = (self.descriptor)(&individual);

        let is_empty = {
            let archive = self.archive.read().unwrap();
            archive.is_empty()
        };

        if is_empty {
            let mut writer = self.archive.write().unwrap();
            writer.push_back(descriptor);
            return vec![1.0];
        }

        let novelty = {
            let archive = self.archive.read().unwrap();
            self.normalized_novelty_score(&descriptor, &archive)
        };

        println!("Novelty: {}", novelty);

        let mut writer = self.archive.write().unwrap();

        println!("Archive size after pop: {}", writer.len());

        if novelty > self.threshold || writer.len() < self.k {
            println!("Adding to archive with novelty: {}", novelty);
            writer.push_back(descriptor);
        }

        vec![novelty]
    }
}

// pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
//     let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
//     let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
//     let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
//     1.0 - dot / (norm_a * norm_b + 1e-8)
// }
