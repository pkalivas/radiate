use crate::{
    BatchFitnessFunction, CosineDistance, EuclideanDistance, FitnessFunction, HammingDistance,
    diversity::Distance, math::knn::KNN,
};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

const DEFAULT_ARCHIVE_SIZE: usize = 1000;
const DEFAULT_K: usize = 15;
const DEFAULT_THRESHOLD: f32 = 0.5;

pub trait Novelty<T>: Send + Sync {
    fn description(&self, member: &T) -> Vec<f32>;
}

impl<T, F> Novelty<T> for F
where
    F: Fn(&T) -> Vec<f32> + Send + Sync,
{
    fn description(&self, member: &T) -> Vec<f32> {
        self(member)
    }
}

#[derive(Clone)]
pub struct NoveltySearch<T> {
    pub behavior: Arc<dyn Novelty<T>>,
    pub archive: Arc<RwLock<VecDeque<Vec<f32>>>>,
    pub k: usize,
    pub threshold: f32,
    pub max_archive_size: usize,
    pub distance_fn: Arc<dyn Distance<Vec<f32>>>,
}

impl<T> NoveltySearch<T> {
    pub fn new<N>(behavior: N) -> Self
    where
        N: Novelty<T> + Send + Sync + 'static,
    {
        NoveltySearch {
            behavior: Arc::new(behavior),
            archive: Arc::new(RwLock::new(VecDeque::new())),
            k: DEFAULT_K,
            threshold: DEFAULT_THRESHOLD,
            max_archive_size: DEFAULT_ARCHIVE_SIZE,
            distance_fn: Arc::new(EuclideanDistance),
        }
    }

    pub fn k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn archive_size(mut self, size: usize) -> Self {
        self.max_archive_size = size;
        self
    }

    pub fn cosine_distance(mut self) -> Self {
        self.distance_fn = Arc::new(CosineDistance);
        self
    }

    pub fn euclidean_distance(mut self) -> Self {
        self.distance_fn = Arc::new(EuclideanDistance);
        self
    }

    pub fn hamming_distance(mut self) -> Self {
        self.distance_fn = Arc::new(HammingDistance);
        self
    }

    fn normalized_novelty_score(
        &self,
        descriptor: &Vec<f32>,
        archive: &mut VecDeque<Vec<f32>>,
    ) -> f32 {
        if archive.is_empty() {
            return 0.5;
        }
        let slice = archive.make_contiguous();

        let mut knn = KNN::new(&slice, Arc::clone(&self.distance_fn));
        let query = knn.query_point(descriptor, self.k);

        let min_distance = query.min_distance;
        let max_distance = query.max_distance;
        if max_distance == min_distance {
            match min_distance {
                _ if min_distance > 0.0 => return 0.5,
                _ => return 0.0,
            }
        }

        let avg_distance = query.average_distance();
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
            let mut archive = self.archive.write().unwrap();
            let result = self.normalized_novelty_score(&description, &mut archive);
            let should_add = result > self.threshold || archive.len() < self.k;

            (result, should_add)
        };

        if should_add {
            let mut writer = self.archive.write().unwrap();

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

impl<T> BatchFitnessFunction<T, f32> for NoveltySearch<T>
where
    T: Send + Sync,
{
    fn evaluate(&self, individuals: Vec<T>) -> Vec<f32> {
        individuals
            .into_iter()
            .map(|ind| self.evaluate_internal(&ind))
            .collect()
    }
}

impl<T> BatchFitnessFunction<&T, f32> for NoveltySearch<T>
where
    T: Send + Sync,
{
    fn evaluate(&self, individuals: Vec<&T>) -> Vec<f32> {
        individuals
            .into_iter()
            .map(|ind| self.evaluate_internal(ind))
            .collect()
    }
}
