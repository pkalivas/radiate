use crate::{FitnessFunction, Score};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

pub trait Novelty<T>: Send + Sync {
    type Descriptor: Send + Sync;

    fn description(&self, phenotype: &T) -> Self::Descriptor;

    fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32;
}

#[derive(Clone)]
pub struct NoveltySearch<T, BD>
where
    BD: Novelty<T> + Send + Sync,
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
    BD: Novelty<T> + Send + Sync,
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

    fn normalized_novelty_score(
        &self,
        descriptor: &BD::Descriptor,
        archive: &VecDeque<BD::Descriptor>,
    ) -> f32 {
        if archive.is_empty() {
            return 1.0;
        }

        let mut min_distance = f32::INFINITY;
        let mut max_distance = f32::NEG_INFINITY;
        let mut distances = archive
            .iter()
            .map(|archived| self.behavior.distance(descriptor, archived))
            .inspect(|&d| {
                max_distance = max_distance.max(d);
                min_distance = min_distance.min(d);
            })
            .collect::<Vec<f32>>();

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
        let description = self.behavior.description(individual);

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
    BD: Novelty<T> + Send + Sync,
    T: Send + Sync,
{
    fn evaluate(&self, individual: T) -> f32 {
        self.evaluate_internal(&individual)
    }
}

impl<T, BD> FitnessFunction<&T, f32> for NoveltySearch<T, BD>
where
    BD: Novelty<T> + Send + Sync,
    T: Send + Sync,
{
    fn evaluate(&self, individual: &T) -> f32 {
        self.evaluate_internal(individual)
    }
}

pub struct FitnessDescriptor<F, T, S>
where
    F: for<'a> FitnessFunction<&'a T, S>,
    S: Into<Score>,
{
    fitness_fn: Arc<F>,
    distance_fn: Arc<dyn Fn(&[f32], &[f32]) -> f32 + Send + Sync>,
    _score_phantom: std::marker::PhantomData<S>,
    _phantom: std::marker::PhantomData<T>,
}

impl<F, T, S> FitnessDescriptor<F, T, S>
where
    F: for<'a> FitnessFunction<&'a T, S> + 'static,
    T: Send + Sync + 'static,
    S: Into<Score> + Send + Sync,
{
    /// Create a new fitness descriptor that uses the output of a fitness function
    /// as the behavioral descriptor. This allows you to use fitness scores
    /// directly as behavioral descriptors for novelty search or diversity measurement.
    pub fn new(fitness_fn: F) -> Self {
        Self {
            fitness_fn: Arc::new(fitness_fn),
            distance_fn: Arc::new(|a, b| {
                if a.len() != b.len() {
                    return f32::INFINITY;
                }
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x - y).powi(2))
                    .sum::<f32>()
                    .sqrt()
            }),
            _score_phantom: std::marker::PhantomData,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_distance_fn(
        self,
        distance_fn: impl Fn(&[f32], &[f32]) -> f32 + Send + Sync + 'static,
    ) -> Self {
        Self {
            distance_fn: Arc::new(distance_fn),
            ..self
        }
    }
}

impl<F, T, S> Novelty<T> for FitnessDescriptor<F, T, S>
where
    F: for<'a> FitnessFunction<&'a T, S> + 'static,
    T: Send + Sync + 'static,
    S: Into<Score> + Send + Sync,
{
    type Descriptor = Vec<f32>;

    fn description(&self, phenotype: &T) -> Self::Descriptor {
        let score = self.fitness_fn.evaluate(phenotype);
        score.into().into()
    }

    fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
        (self.distance_fn)(a, b)
    }
}
