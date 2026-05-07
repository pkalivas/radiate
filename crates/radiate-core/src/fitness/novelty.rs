use crate::{
    BatchFitnessFunction, BatchedFn, CosineDistance, EuclideanDistance, FitnessFunction,
    HammingDistance, diversity::Distance, math::knn::KNN,
};
use radiate_utils::WindowBuffer;
use std::sync::{Arc, RwLock};

const DEFAULT_ARCHIVE_SIZE: usize = 1000;
const DEFAULT_K: usize = 15;
const DEFAULT_THRESHOLD: f32 = 0.5;

pub trait Novelty<T>: Send + Sync {
    fn description(&self, member: &T) -> Vec<f32>;

    /// Compute descriptors for a whole batch. Default fans out to `description`.
    /// Override this on your own concrete `Novelty` impl if you can vectorise the
    /// batch path (shared setup, SIMD, GPU, etc.) — the closure blanket impl
    /// always takes the default.
    fn batch_description(&self, members: &[T]) -> Vec<Vec<f32>> {
        members.iter().map(|m| self.description(m)).collect()
    }
}

impl<T, F> Novelty<T> for F
where
    F: Fn(&T) -> Vec<f32> + Send + Sync,
{
    fn description(&self, member: &T) -> Vec<f32> {
        self(member)
    }
}

impl<T, F> Novelty<T> for BatchedFn<F>
where
    F: Fn(&[T]) -> Vec<Vec<f32>> + Send + Sync,
{
    fn description(&self, member: &T) -> Vec<f32> {
        (self.0)(std::slice::from_ref(member))
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    fn batch_description(&self, members: &[T]) -> Vec<Vec<f32>> {
        (self.0)(members)
    }
}

#[derive(Clone)]
pub struct NoveltySearch<T> {
    pub behavior: Arc<dyn Novelty<T>>,
    pub archive: Arc<RwLock<WindowBuffer<Vec<f32>>>>,
    pub k: usize,
    pub threshold: f32,
    pub distance_fn: Arc<dyn Distance<Vec<f32>>>,
}

impl<T> NoveltySearch<T> {
    pub fn new<N>(behavior: N) -> Self
    where
        N: Novelty<T> + Send + Sync + 'static,
    {
        NoveltySearch {
            behavior: Arc::new(behavior),
            archive: Arc::new(RwLock::new(WindowBuffer::with_window(DEFAULT_ARCHIVE_SIZE))),
            k: DEFAULT_K,
            threshold: DEFAULT_THRESHOLD,
            distance_fn: Arc::new(EuclideanDistance),
        }
    }

    /// Construct from a batch-shaped descriptor closure.
    /// Equivalent to `NoveltySearch::new(BatchedFn(f))`.
    pub fn from_batch_fn<F>(f: F) -> Self
    where
        F: Fn(&[T]) -> Vec<Vec<f32>> + Send + Sync + 'static,
        T: 'static,
    {
        Self::new(BatchedFn(f))
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
        self.archive = Arc::new(RwLock::new(WindowBuffer::with_window(size)));
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

    fn novelty_score(&self, descriptor: &Vec<f32>, archive: &WindowBuffer<Vec<f32>>) -> f32 {
        let slice = archive.values();
        let mut knn = KNN::new(slice, Arc::clone(&self.distance_fn));
        let query = knn.query_point(descriptor, self.k);

        let min_dist = query.min_distance;
        let max_dist = query.max_distance;
        let range = max_dist - min_dist;

        if range < f32::EPSILON {
            return if min_dist < f32::EPSILON { 0.0 } else { 0.5 };
        }

        let avg_dist = query.average_distance();
        (avg_dist - min_dist) / range
    }

    fn evaluate_internal(&self, individual: &T) -> f32 {
        let description = self.behavior.description(individual);
        let mut archive = self.archive.write().unwrap();

        if archive.is_empty() {
            archive.push(description);
            return 0.5;
        }

        let novelty = self.novelty_score(&description, &archive);
        if novelty > self.threshold || archive.len() < self.k {
            archive.push(description);
        }

        novelty
    }

    fn evaluate_batch_internal(&self, individuals: &[T]) -> Vec<f32> {
        let descriptions = self.behavior.batch_description(individuals);
        let mut archive = self.archive.write().unwrap();

        if archive.is_empty() {
            let result = vec![0.5; descriptions.len()];
            for desc in descriptions {
                archive.push(desc);
            }

            return result;
        }

        // Score every descriptor against the same pre-batch archive snapshot —
        // no archive mutations happen during scoring, so individual-N's score
        // does not depend on its position in the batch.
        let mut scores = Vec::with_capacity(descriptions.len());
        for desc in descriptions.into_iter() {
            let score = self.novelty_score(&desc, &archive);

            if score > self.threshold || archive.len() < self.k {
                archive.push(desc);
            }

            scores.push(score);
        }

        scores
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
        self.evaluate_batch_internal(&individuals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BatchFitnessFunction, FitnessFunction};

    fn make_ns(k: usize, threshold: f32) -> NoveltySearch<Vec<f32>> {
        NoveltySearch::new(|v: &Vec<f32>| v.clone())
            .k(k)
            .threshold(threshold)
            .archive_size(100)
    }

    fn seed(ns: &NoveltySearch<Vec<f32>>, points: impl IntoIterator<Item = Vec<f32>>) {
        let mut archive = ns.archive.write().unwrap();
        for p in points {
            archive.push(p);
        }
    }

    fn archive_view_len(ns: &NoveltySearch<Vec<f32>>) -> usize {
        ns.archive.read().unwrap().values().len()
    }

    fn eval(ns: &NoveltySearch<Vec<f32>>, v: Vec<f32>) -> f32 {
        <NoveltySearch<Vec<f32>> as FitnessFunction<Vec<f32>, f32>>::evaluate(ns, v)
    }

    fn eval_batch(ns: &NoveltySearch<Vec<f32>>, vs: Vec<Vec<f32>>) -> Vec<f32> {
        <NoveltySearch<Vec<f32>> as BatchFitnessFunction<Vec<f32>, f32>>::evaluate(ns, vs)
    }

    #[test]
    fn empty_archive_single_eval_scores_half_and_seeds_one_entry() {
        let ns = make_ns(3, 0.5);
        let score = eval(&ns, vec![1.0, 0.0]);
        assert_eq!(score, 0.5);
        assert_eq!(archive_view_len(&ns), 1);
    }

    #[test]
    fn empty_archive_batch_eval_scores_half_and_seeds_every_entry() {
        let ns = make_ns(3, 0.5);
        let scores = eval_batch(
            &ns,
            vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0], vec![4.0]],
        );

        assert_eq!(scores, vec![0.5; 5]);
        assert_eq!(archive_view_len(&ns), 5);
    }

    #[test]
    fn identical_to_sole_archive_point_scores_zero() {
        let ns = make_ns(3, 0.99);
        seed(&ns, [vec![1.0, 0.0]]);

        // distance is clamped to 1e-12 inside KNN, which is < f32::EPSILON,
        // so the degenerate branch returns 0.0.
        let score = eval(&ns, vec![1.0, 0.0]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn different_from_sole_archive_point_scores_half_neutral() {
        let ns = make_ns(3, 0.99);
        seed(&ns, [vec![0.0]]);

        // single archive point => min == max, range == 0, min > epsilon => 0.5
        let score = eval(&ns, vec![5.0]);
        assert_eq!(score, 0.5);
    }

    #[test]
    fn bootstrap_admits_first_k_individuals_under_strict_threshold() {
        // threshold 0.99 means scores effectively can't pass on merit.
        // archive.len() < k path must still admit the first k.
        let ns = make_ns(3, 0.99);

        for _ in 0..3 {
            eval(&ns, vec![1.0]);
        }
        assert_eq!(archive_view_len(&ns), 3);

        // 4th identical individual: archive.len() == k, score won't beat 0.99 → not added.
        eval(&ns, vec![1.0]);
        assert_eq!(archive_view_len(&ns), 3);
    }

    #[test]
    fn post_bootstrap_score_below_threshold_does_not_admit() {
        let ns = make_ns(2, 0.2);
        // 3 seeded points → past bootstrap (archive.len() >= k=2).
        seed(&ns, [vec![0.0], vec![5.0], vec![10.0]]);

        // query=[2.0]: 2-NN are [0.0]@2 and [5.0]@3, global_max=8 (to [10.0]).
        // avg=2.5, range=6 → score = 0.5/6 ≈ 0.0833 < 0.2 → not added.
        let score = eval(&ns, vec![2.0]);
        assert!(
            (score - 0.083_333).abs() < 1e-4,
            "expected ≈0.0833, got {score}"
        );
        assert_eq!(archive_view_len(&ns), 3);
    }

    #[test]
    fn novelty_score_matches_normalization_formula() {
        // threshold = -1 so admission never filters; we want to inspect the score itself.
        let ns = make_ns(2, -1.0);
        seed(&ns, [vec![0.0], vec![5.0], vec![10.0]]);

        // query=[12.0]: 2-NN are [10.0]@2 and [5.0]@7, global_max=12 (to [0.0]).
        // avg=4.5, range=10 → score = 2.5/10 = 0.25 exact.
        let score = eval(&ns, vec![12.0]);
        assert!((score - 0.25).abs() < 1e-5, "expected 0.25, got {score}");
    }

    #[test]
    fn novelty_score_with_k_equal_to_archive_size_uses_all_points() {
        let ns = make_ns(3, -1.0);
        seed(&ns, [vec![0.0], vec![5.0], vec![10.0]]);

        // k >= n branch: cluster contains all archive points sorted ascending.
        // query=[4.0] distances: 4, 1, 6 → min=1, max=6, avg=11/3.
        // score = (11/3 - 1) / 5 = (8/3) / 5 = 8/15 ≈ 0.5333.
        let score = eval(&ns, vec![4.0]);
        assert!(
            (score - 8.0 / 15.0).abs() < 1e-5,
            "expected 8/15 ≈ 0.5333, got {score}"
        );
    }

    #[test]
    fn novelty_score_always_in_unit_interval() {
        let ns = make_ns(2, -1.0);
        seed(&ns, [vec![0.0], vec![5.0], vec![10.0]]);

        for x in [-100.0, -1.0, 0.0, 2.5, 5.0, 7.5, 10.0, 12.0, 100.0] {
            // fresh archive each iteration so writes from earlier queries don't drift.
            let ns = make_ns(2, -1.0);
            seed(&ns, [vec![0.0], vec![5.0], vec![10.0]]);

            let score = eval(&ns, vec![x]);
            assert!(
                (0.0..=1.0).contains(&score),
                "score {score} out of [0,1] for x={x}"
            );
        }
    }

    #[test]
    fn score_above_threshold_admits_to_archive() {
        let ns = make_ns(2, 0.2);
        seed(&ns, [vec![0.0], vec![5.0], vec![10.0]]);
        assert_eq!(archive_view_len(&ns), 3);

        // query=[12.0] → score 0.25 > 0.2 → admitted.
        let score = eval(&ns, vec![12.0]);
        assert!(score > 0.2, "expected > threshold, got {score}");
        assert_eq!(archive_view_len(&ns), 4);
    }

    #[test]
    fn archive_window_caps_at_configured_size() {
        // archive_size=5, threshold=-1 (always admit).
        let ns = NoveltySearch::new(|v: &Vec<f32>| v.clone())
            .k(1)
            .threshold(-1.0)
            .archive_size(5);

        for i in 0..40 {
            eval(&ns, vec![i as f32 * 100.0]);
        }

        // The k-NN sees archive.values(), which is the live window.
        let archive = ns.archive.read().unwrap();
        assert!(
            archive.values().len() <= 5,
            "archive view exceeds window cap: {}",
            archive.values().len()
        );
    }

    #[test]
    fn fitness_function_ref_variant_evaluates_and_admits() {
        let ns = make_ns(1, 0.5);
        let ind = vec![1.0, 0.0];
        let score = eval(&ns, ind);
        assert_eq!(score, 0.5);
        assert_eq!(archive_view_len(&ns), 1);
    }

    #[test]
    fn batch_eval_returns_one_score_per_individual() {
        let ns = make_ns(3, 0.5);
        let scores = eval_batch(&ns, vec![vec![0.0], vec![5.0], vec![10.0]]);
        assert_eq!(scores.len(), 3);
        for (i, &s) in scores.iter().enumerate() {
            assert!((0.0..=1.0).contains(&s), "scores[{i}] = {s} out of [0,1]");
        }
    }

    #[test]
    fn batch_eval_admits_via_running_archive_size_for_bootstrap() {
        // Scoring is against a frozen pre-batch snapshot, but the admission pass
        // walks the batch with the live archive size so bootstrap (`archive.len()
        // < k`) still works mid-batch when the pre-batch archive is undersized.
        let ns = make_ns(2, -1.0); // threshold=-1 → score-based admission also passes.
        seed(&ns, [vec![0.0], vec![10.0]]);
        let initial = archive_view_len(&ns);

        let scores = eval_batch(&ns, vec![vec![5.0], vec![20.0], vec![-5.0]]);
        assert_eq!(scores.len(), 3);
        assert_eq!(archive_view_len(&ns), initial + 3);
    }

    #[test]
    fn batch_eval_does_not_score_against_intra_batch_additions() {
        // If the batch were "online" (admitting earlier members before scoring
        // later ones), then a duplicate entry later in the batch would see its
        // earlier copy as a near-zero-distance neighbour and score ~0.
        // True batch should score the duplicate against only the pre-batch
        // archive — yielding the same score as the original.
        let ns = make_ns(2, -1.0);
        seed(&ns, [vec![0.0], vec![10.0]]);

        let scores = eval_batch(&ns, vec![vec![5.0], vec![5.0]]);
        assert_eq!(scores.len(), 2);
        assert!(
            (scores[0] - scores[1]).abs() < 1e-6,
            "duplicate batch members should score identically: {scores:?}"
        );
    }

    #[test]
    fn from_batch_fn_routes_batch_through_user_closure_and_falls_back_per_item() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let batch_calls = Arc::new(AtomicUsize::new(0));
        let total_seen = Arc::new(AtomicUsize::new(0));

        let ns: NoveltySearch<Vec<f32>> = {
            let batch_calls = Arc::clone(&batch_calls);
            let total_seen = Arc::clone(&total_seen);
            NoveltySearch::from_batch_fn(move |members: &[Vec<f32>]| {
                batch_calls.fetch_add(1, Ordering::Relaxed);
                total_seen.fetch_add(members.len(), Ordering::Relaxed);
                members.iter().map(|v| v.clone()).collect()
            })
            .k(3)
            .threshold(0.5)
            .archive_size(100)
        };

        // Single-eval routes through the per-item fallback, which calls the
        // batch closure with a 1-element slice.
        let _ = eval(&ns, vec![1.0, 0.0]);
        assert_eq!(batch_calls.load(Ordering::Relaxed), 1);
        assert_eq!(total_seen.load(Ordering::Relaxed), 1);

        // Batch eval calls the closure once with the full slice — the fast path.
        let _ = eval_batch(&ns, vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]]);
        assert_eq!(batch_calls.load(Ordering::Relaxed), 2);
        assert_eq!(total_seen.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn clone_shares_archive_with_original() {
        let ns = make_ns(3, 0.5);
        let twin = ns.clone();

        eval(&ns, vec![1.0]);
        eval(&ns, vec![2.0]);

        // Same Arc<RwLock<...>> backing both handles.
        assert_eq!(archive_view_len(&twin), 2);
    }

    #[test]
    fn cosine_distance_identical_direction_scores_zero_in_degenerate_case() {
        let ns = NoveltySearch::new(|v: &Vec<f32>| v.clone())
            .k(3)
            .threshold(0.99)
            .archive_size(100)
            .cosine_distance();
        seed(&ns, [vec![1.0, 0.0]]);

        // Same direction, different magnitude → cosine distance 0 (clamped to 1e-12).
        let score = eval(&ns, vec![100.0, 0.0]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn concurrent_evaluation_does_not_panic_or_deadlock() {
        use std::thread;

        let ns = Arc::new(
            NoveltySearch::new(|v: &Vec<f32>| v.clone())
                .k(5)
                .threshold(0.3)
                .archive_size(200),
        );

        let handles: Vec<_> = (0..8)
            .map(|i| {
                let ns = Arc::clone(&ns);
                thread::spawn(move || {
                    for j in 0..50 {
                        let v = (i * 50 + j) as f32;
                        eval(&ns, vec![v, v * 0.5]);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().expect("worker thread panicked");
        }

        // 8 threads × 50 evals = 400 attempts; capped by archive_size=200.
        let archive = ns.archive.read().unwrap();
        assert!(archive.values().len() <= 200);
    }
}
