use crate::{FitnessFunction, Score};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

pub trait Novelty<T> {
    fn description(&self, phenotype: &T) -> Vec<f32>;
}

#[derive(Clone)]
pub struct NoveltySearch<T> {
    pub behavior: Arc<dyn Novelty<T> + Send + Sync>,
    pub archive: Arc<RwLock<VecDeque<Vec<f32>>>>,
    pub k: usize,
    pub threshold: f32,
    pub max_archive_size: usize,
    pub distance_fn: Arc<dyn Fn(&[f32], &[f32]) -> f32 + Send + Sync>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> NoveltySearch<T> {
    pub fn new<N>(behavior: N, k: usize, threshold: f32) -> Self
    where
        N: Novelty<T> + Send + Sync + 'static,
    {
        NoveltySearch {
            behavior: Arc::new(behavior),
            archive: Arc::new(RwLock::new(VecDeque::new())),
            k,
            threshold,
            max_archive_size: 1000,
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
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_max_archive_size(mut self, size: usize) -> Self {
        self.max_archive_size = size;
        self
    }

    pub fn cosine_distance(mut self) -> Self {
        self.distance_fn = Arc::new(|a, b| {
            let dot_product = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
            let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            1.0 - (dot_product / (norm_a * norm_b))
        });
        self
    }

    pub fn euclidean_distance(mut self) -> Self {
        self.distance_fn = Arc::new(|a, b| {
            if a.len() != b.len() {
                return f32::INFINITY;
            }
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f32>()
                .sqrt()
        });
        self
    }

    pub fn hamming_distance(mut self) -> Self {
        self.distance_fn = Arc::new(|a, b| {
            if a.len() != b.len() {
                return f32::INFINITY;
            }
            a.iter().zip(b.iter()).filter(|(x, y)| x != y).count() as f32
        });
        self
    }

    fn normalized_novelty_score(&self, descriptor: &Vec<f32>, archive: &VecDeque<Vec<f32>>) -> f32 {
        if archive.is_empty() {
            return 0.5;
        }

        let mut min_distance = f32::INFINITY;
        let mut max_distance = f32::NEG_INFINITY;
        let mut distances = archive
            .iter()
            .map(|archived| (self.distance_fn)(&descriptor, archived))
            .inspect(|&d| {
                max_distance = max_distance.max(d);
                min_distance = min_distance.min(d);
            })
            .collect::<Vec<f32>>();

        if max_distance == min_distance {
            if min_distance == 0.0 {
                return 0.0;
            }

            if min_distance > 0.0 {
                return 0.5;
            }

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
            return 0.5;
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

impl<T> FitnessFunction<T, f32> for NoveltySearch<T>
where
    T: Send + Sync,
{
    fn evaluate(&self, individual: T) -> f32 {
        self.evaluate_internal(&individual)
    }
}

impl<T> FitnessFunction<&T, f32> for NoveltySearch<T>
where
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
            _score_phantom: std::marker::PhantomData,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, T, S> Novelty<T> for FitnessDescriptor<F, T, S>
where
    F: for<'a> FitnessFunction<&'a T, S> + 'static,
    T: Send + Sync + 'static,
    S: Into<Score> + Send + Sync,
{
    fn description(&self, phenotype: &T) -> Vec<f32> {
        let score = self.fitness_fn.evaluate(phenotype);
        score.into().into()
    }
}
