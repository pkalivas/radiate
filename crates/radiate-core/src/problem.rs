use super::{Chromosome, Codec, Genotype, Score};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

pub trait FitnessFunction<T, S = f32>: Send + Sync
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

/// Multi-objective fitness function that combines multiple objectives
pub struct MultiObjectiveFitness<T, S> {
    objectives: Vec<Arc<dyn for<'a> FitnessFunction<&'a T, S>>>,
    weights: Vec<f32>,
}

impl<T, S> MultiObjectiveFitness<T, S>
where
    S: Into<Score> + Clone,
{
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_objective<F>(mut self, fitness_fn: F, weight: f32) -> Self
    where
        F: for<'a> FitnessFunction<&'a T, S> + 'static,
    {
        self.objectives.push(Arc::new(fitness_fn));
        self.weights.push(weight);
        self
    }

    pub fn add_objective_fn(
        mut self,
        fitness_fn: impl for<'a> FitnessFunction<&'a T, S> + 'static,
        weight: f32,
    ) -> Self
    where
        S: Into<Score>,
    {
        self.objectives.push(Arc::new(fitness_fn));
        self.weights.push(weight);
        self
    }
}

impl<T> FitnessFunction<T> for MultiObjectiveFitness<T, f32> {
    fn evaluate(&self, individual: T) -> f32 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        for (objective, weight) in self.objectives.iter().zip(&self.weights) {
            let score = objective.evaluate(&individual);
            total_score += score * weight;
            total_weight += weight;
        }

        total_score / total_weight.max(1e-6)
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
pub struct NoveltySearch<T, BD>
where
    BD: BehavioralDescriptor<T> + Send + Sync,
{
    pub behavior: Arc<BD>,
    pub archive: Arc<RwLock<VecDeque<BD::Descriptor>>>,
    pub k: usize,
    pub threshold: f32,
    pub max_archive_size: usize,
    __phantom: std::marker::PhantomData<T>,
}

impl<T, BD> NoveltySearch<T, BD>
where
    BD: BehavioralDescriptor<T> + Send + Sync,
{
    pub fn new(behavior: BD, k: usize, threshold: f32) -> Self {
        NoveltySearch {
            behavior: Arc::new(behavior),
            archive: Arc::new(RwLock::new(VecDeque::new())),
            k,
            threshold,
            max_archive_size: 1000,
            __phantom: std::marker::PhantomData,
        }
    }

    pub fn with_max_archive_size(mut self, size: usize) -> Self {
        self.max_archive_size = size;
        self
    }

    pub fn get_archive(&self) -> VecDeque<BD::Descriptor>
    where
        BD::Descriptor: Clone,
    {
        (*self.archive.read().unwrap()).clone()
    }

    fn normalized_novelty_score(
        &self,
        descriptor: &BD::Descriptor,
        archive: &VecDeque<BD::Descriptor>,
    ) -> f32 {
        if archive.is_empty() {
            return 1.0;
        }

        let mut distances = archive
            .iter()
            .map(|archived| self.behavior.distance(descriptor, archived))
            .collect::<Vec<f32>>();

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

    fn evaluate_internal(&self, individual: &T) -> f32 {
        let description = self.behavior.extract_descriptor(&individual);

        let is_empty = {
            let archive = self.archive.read().unwrap();
            archive.is_empty()
        };

        if is_empty {
            let mut writer = self.archive.write().unwrap();
            writer.push_back(description);
            return 1.0;
        }

        let (novelty, should_add) = {
            let archive = self.archive.read().unwrap();
            let result = self.normalized_novelty_score(&description, &archive);
            let should_add = result > self.threshold || archive.len() < self.k;

            (result, should_add)
        };

        let mut writer = self.archive.write().unwrap();

        if should_add {
            writer.push_back(description);
            while writer.len() > self.max_archive_size {
                writer.pop_front();
            }
        }

        novelty
    }
}

impl<T, BD> FitnessFunction<T, f32> for NoveltySearch<T, BD>
where
    BD: BehavioralDescriptor<T> + Send + Sync,
    T: Send + Sync,
{
    fn evaluate(&self, individual: T) -> f32 {
        self.evaluate_internal(&individual)
    }
}

impl<T, BD> FitnessFunction<&T, f32> for NoveltySearch<T, BD>
where
    BD: BehavioralDescriptor<T> + Send + Sync,
    T: Send + Sync,
{
    fn evaluate(&self, individual: &T) -> f32 {
        self.evaluate_internal(individual)
    }
}

pub trait BehavioralDescriptor<T>: Send + Sync {
    type Descriptor: Send + Sync;

    /// Extract a behavioral descriptor from a phenotype
    fn extract_descriptor(&self, phenotype: &T) -> Self::Descriptor;

    /// Calculate distance between two behavioral descriptors
    fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32;
}
