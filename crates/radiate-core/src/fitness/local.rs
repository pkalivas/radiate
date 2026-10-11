use crate::{BatchFitnessFunction, FitnessFunction, Objective, Score, pareto};
use std::sync::Arc;

const DEFAULT_STEPS: usize = 10;

type NeighborFn<T> = Arc<dyn Fn(&T) -> T + Send + Sync>;

/// A fitness function that refines each individual with a short hill climb before scoring it.
///
/// [LocalSearch] wraps an inner fitness function and a neighbor function. Each evaluation
/// scores the individual, then up to `steps` times builds a neighbor of the current best,
/// scores it, and moves to it when its score dominates the current best's under the
/// configured [Objective]. The returned score is the best one found.
///
/// The population keeps the original genotype - only the score reflects the refinement.
/// This means individuals are credited for how good they *could* become nearby
/// (a Baldwinian memetic search), which smooths rugged landscapes without collapsing diversity.
///
/// Each evaluation costs `steps + 1` calls to the inner fitness function. Individuals that
/// survive unchanged keep their score, so they are not searched again.
///
/// # Generic Parameters
/// - `T`: The type of individual being evaluated
/// - `F`: The inner fitness function
///
/// # Examples
///
/// ```rust
/// use radiate_core::fitness::{FitnessFunction, LocalSearch};
///
/// // Maximize -(x - 3)^2, stepping x by +0.5 each time.
/// let search = LocalSearch::new(
///     |x: f32| -(x - 3.0).powi(2),
///     |x: &f32| x + 0.5,
/// )
/// .steps(4);
///
/// // 1.0 -> 1.5 -> 2.0 -> 2.5 -> 3.0
/// let score: f32 = search.evaluate(1.0);
/// assert_eq!(score, 0.0);
/// ```
pub struct LocalSearch<T, F> {
    fitness: F,
    neighbor: NeighborFn<T>,
    steps: usize,
    objective: Objective,
}

impl<T, F> LocalSearch<T, F> {
    pub fn new<N>(fitness: F, neighbor: N) -> Self
    where
        N: Fn(&T) -> T + Send + Sync + 'static,
    {
        LocalSearch {
            fitness,
            neighbor: Arc::new(neighbor),
            steps: DEFAULT_STEPS,
            objective: Objective::default(),
        }
    }

    /// The number of neighbors tried per individual.
    pub fn steps(mut self, steps: usize) -> Self {
        self.steps = steps;
        self
    }

    /// The objective used to decide whether a neighbor improves on the current best.
    /// This should match the objective the engine is built with.
    pub fn objective(mut self, objective: impl Into<Objective>) -> Self {
        self.objective = objective.into();
        self
    }

    pub fn minimizing(self) -> Self {
        self.objective(Objective::Single(crate::Optimize::Minimize))
    }

    pub fn maximizing(self) -> Self {
        self.objective(Objective::Single(crate::Optimize::Maximize))
    }

    fn improves<S>(&self, candidate: &S, best: &S) -> bool
    where
        S: Into<Score> + Clone,
    {
        let candidate: Score = candidate.clone().into();
        let best: Score = best.clone().into();
        pareto::dominance(&candidate, &best, &self.objective)
    }
}

impl<T, F> Clone for LocalSearch<T, F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            fitness: self.fitness.clone(),
            neighbor: Arc::clone(&self.neighbor),
            steps: self.steps,
            objective: self.objective.clone(),
        }
    }
}

impl<T, S, F> FitnessFunction<T, S> for LocalSearch<T, F>
where
    T: Clone + Send + Sync,
    S: Into<Score> + Clone,
    F: FitnessFunction<T, S>,
{
    fn evaluate(&self, individual: T) -> S {
        let mut best_score = self.fitness.evaluate(individual.clone());
        let mut current = individual;

        for _ in 0..self.steps {
            let candidate = (self.neighbor)(&current);
            let score = self.fitness.evaluate(candidate.clone());

            if self.improves(&score, &best_score) {
                best_score = score;
                current = candidate;
            }
        }

        best_score
    }
}

impl<T, S, F> FitnessFunction<&T, S> for LocalSearch<T, F>
where
    T: Send + Sync,
    S: Into<Score> + Clone,
    F: for<'a> FitnessFunction<&'a T, S>,
{
    fn evaluate(&self, individual: &T) -> S {
        let mut best_score = self.fitness.evaluate(individual);
        let mut current: Option<T> = None;

        for _ in 0..self.steps {
            let candidate = (self.neighbor)(current.as_ref().unwrap_or(individual));
            let score = self.fitness.evaluate(&candidate);

            if self.improves(&score, &best_score) {
                best_score = score;
                current = Some(candidate);
            }
        }

        best_score
    }
}

/// Every step builds one neighbor per individual and scores them all with a single
/// call to the inner batch function, so a batch costs `steps + 1` inner calls in total.
impl<T, S, F> BatchFitnessFunction<T, S> for LocalSearch<T, F>
where
    T: Clone + Send + Sync,
    S: Into<Score> + Clone,
    F: BatchFitnessFunction<T, S>,
{
    fn evaluate(&self, individuals: Vec<T>) -> Vec<S> {
        let mut best_scores = self.fitness.evaluate(individuals.clone());
        let mut current = individuals;

        for _ in 0..self.steps {
            let candidates = current
                .iter()
                .map(|ind| (self.neighbor)(ind))
                .collect::<Vec<T>>();
            let scores = self.fitness.evaluate(candidates.clone());

            for (i, (candidate, score)) in candidates.into_iter().zip(scores).enumerate() {
                if i < best_scores.len() && self.improves(&score, &best_scores[i]) {
                    best_scores[i] = score;
                    current[i] = candidate;
                }
            }
        }

        best_scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BatchedFn, Optimize};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn peak_at_three(x: f32) -> f32 {
        -(x - 3.0).powi(2)
    }

    fn eval<F>(search: &LocalSearch<f32, F>, x: f32) -> f32
    where
        F: FitnessFunction<f32, f32>,
    {
        <LocalSearch<f32, F> as FitnessFunction<f32, f32>>::evaluate(search, x)
    }

    #[test]
    fn zero_steps_returns_the_inner_score() {
        let search = LocalSearch::new(peak_at_three, |x: &f32| x + 1.0).steps(0);
        assert_eq!(eval(&search, 0.0), peak_at_three(0.0));
    }

    #[test]
    fn climbs_toward_the_peak_when_maximizing() {
        let search = LocalSearch::new(peak_at_three, |x: &f32| x + 0.5).steps(4);
        assert_eq!(eval(&search, 1.0), 0.0);
    }

    #[test]
    fn stops_moving_once_neighbors_get_worse() {
        // From 1.0 with +1.0 steps: 2.0 (-1), 3.0 (0), then 4.0 (-1) is rejected,
        // and every later neighbor is built from 3.0 again.
        let search = LocalSearch::new(peak_at_three, |x: &f32| x + 1.0).steps(10);
        assert_eq!(eval(&search, 1.0), 0.0);
    }

    #[test]
    fn never_returns_worse_than_the_starting_score() {
        let search = LocalSearch::new(peak_at_three, |x: &f32| x + 1.0).steps(5);
        assert_eq!(eval(&search, 3.0), 0.0);
    }

    #[test]
    fn minimizing_flips_which_neighbors_are_accepted() {
        let search = LocalSearch::new(|x: f32| x, |x: &f32| x - 1.0)
            .minimizing()
            .steps(3);
        assert_eq!(eval(&search, 10.0), 7.0);

        let search = LocalSearch::new(|x: f32| x, |x: &f32| x - 1.0)
            .maximizing()
            .steps(3);
        assert_eq!(eval(&search, 10.0), 10.0);
    }

    #[test]
    fn calls_inner_fitness_steps_plus_one_times() {
        let calls = Arc::new(AtomicUsize::new(0));
        let search = {
            let calls = Arc::clone(&calls);
            LocalSearch::new(
                move |x: f32| {
                    calls.fetch_add(1, Ordering::Relaxed);
                    x
                },
                |x: &f32| x + 1.0,
            )
            .steps(7)
        };

        eval(&search, 0.0);
        assert_eq!(calls.load(Ordering::Relaxed), 8);
    }

    #[test]
    fn ref_variant_searches_without_cloning_the_input() {
        let search = LocalSearch::new(|x: &f32| -(x - 3.0).powi(2), |x: &f32| x + 0.5).steps(4);
        let start = 1.0;
        let score = <LocalSearch<f32, _> as FitnessFunction<&f32, f32>>::evaluate(&search, &start);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn multi_objective_only_accepts_dominating_neighbors() {
        // Neighbor improves the first objective but worsens the second: not a dominating move.
        let objective = Objective::Multi(vec![Optimize::Maximize, Optimize::Maximize]);
        let search = LocalSearch::new(|x: f32| vec![x, -x], |x: &f32| x + 1.0)
            .objective(objective.clone())
            .steps(5);
        let score = <LocalSearch<f32, _> as FitnessFunction<f32, Vec<f32>>>::evaluate(&search, 0.0);
        assert_eq!(score, vec![0.0, 0.0]);

        // Neighbor improves both objectives: every step is accepted.
        let search = LocalSearch::new(|x: f32| vec![x, x], |x: &f32| x + 1.0)
            .objective(objective)
            .steps(5);
        let score = <LocalSearch<f32, _> as FitnessFunction<f32, Vec<f32>>>::evaluate(&search, 0.0);
        assert_eq!(score, vec![5.0, 5.0]);
    }

    #[test]
    fn batch_refines_each_individual_independently() {
        let search = LocalSearch::new(
            BatchedFn(|xs: &[f32]| xs.iter().map(|&x| peak_at_three(x)).collect()),
            |x: &f32| x + 1.0,
        )
        .steps(5);

        // 1.0 climbs to 3.0; 3.0 is already at the peak; 5.0 only moves away from it.
        let scores = <LocalSearch<f32, _> as BatchFitnessFunction<f32, f32>>::evaluate(
            &search,
            vec![1.0, 3.0, 5.0],
        );
        assert_eq!(scores, vec![0.0, 0.0, -4.0]);
    }

    #[test]
    fn batch_calls_inner_once_per_step_with_the_whole_batch() {
        let calls = Arc::new(AtomicUsize::new(0));
        let sizes = Arc::new(AtomicUsize::new(0));
        let search = {
            let calls = Arc::clone(&calls);
            let sizes = Arc::clone(&sizes);
            LocalSearch::new(
                BatchedFn(move |xs: &[f32]| {
                    calls.fetch_add(1, Ordering::Relaxed);
                    sizes.fetch_add(xs.len(), Ordering::Relaxed);
                    xs.to_vec()
                }),
                |x: &f32| x + 1.0,
            )
            .steps(3)
        };

        let _ = <LocalSearch<f32, _> as BatchFitnessFunction<f32, f32>>::evaluate(
            &search,
            vec![0.0; 6],
        );
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        assert_eq!(sizes.load(Ordering::Relaxed), 24);
    }

    #[test]
    fn batch_and_single_agree() {
        let single = LocalSearch::new(peak_at_three, |x: &f32| x + 0.5).steps(6);
        let batch = LocalSearch::new(
            BatchedFn(|xs: &[f32]| xs.iter().map(|&x| peak_at_three(x)).collect()),
            |x: &f32| x + 0.5,
        )
        .steps(6);

        let inputs = vec![-2.0, 0.0, 1.25, 3.0, 4.5];
        let expected = inputs.iter().map(|&x| eval(&single, x)).collect::<Vec<_>>();
        let scores =
            <LocalSearch<f32, _> as BatchFitnessFunction<f32, f32>>::evaluate(&batch, inputs);
        assert_eq!(scores, expected);
    }
}
