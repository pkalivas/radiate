//! # Engine Iterator System
//!
//! This module provides a comprehensive iterator system for genetic algorithm engines,
//! enabling fine-grained control over evolutionary execution with various termination
//! conditions and monitoring capabilities.
//!
//! The iterator system extends the basic `Engine` trait with powerful methods for:
//! - **Termination Control**: Generation limits, time limits, score thresholds, and convergence detection
//! - **Monitoring**: Logging, progress tracking, and performance metrics
//! - **Adaptive Execution**: Stagnation detection and early stopping
//! - **Composable Limits**: Combining multiple termination conditions
//!
//! # Key Components
//!
//! - **EngineIterator**: Basic iterator wrapper around any engine
//! - **EngineIteratorExt**: Extension trait providing termination and monitoring methods
//! - **Specialized Iterators**: Various iterator types for different termination strategies
//! - **Limit System**: Flexible limit specification and combination

use crate::{Generation, Limit, init_logging};
use radiate_core::{Chromosome, Engine, Objective, Optimize, Score};
use std::{collections::VecDeque, time::Duration};
use tracing::info;

/// A basic iterator wrapper around any engine that implements the [Engine] trait.
///
/// [EngineIterator] provides a simple way to iterate over the generations produced
/// by an engine, yielding one generation per iteration. This is the foundation
/// for more sophisticated iteration patterns through the extension trait.
///
/// The [EngineIterator] is an 'infinite' iterator, meaning it will continue to
/// produce generations until explicitly terminated so always provide termination conditions,
/// whether through traditional means (like `.last()`, `.take()`, or custom predicates).
///
/// # Generic Parameters
///
/// - `E`: The engine type that this iterator wraps
///
/// # Examples
///
/// ```rust,ignore
/// use radiate_engines::*;
///
/// let engine = GeneticEngine::builder()
///     .codec(FloatCodec::vector(5, 0.0..1.0))
///     .fitness_fn(|x: Vec<f32>| x.iter().sum::<f32>())
///     .build();
///
/// // Basic iteration
/// for generation in engine.iter() {
///     println!("Generation {}: Score = {}",
///              generation.index(), generation.score().as_f32());
///
///     if generation.index() > 10 {
///         break;
///     }
/// }
///
/// // Basic iterator through functional methods - notice the call to last().
/// // This will retrieve the last generation after 10 iterations.
/// let last_generation = engine.iter().take(10).last();
/// ```
///
/// # Design
///
/// This iterator is intentionally simple, providing the basic iteration capability
/// while delegating advanced functionality to the extension trait. This separation
/// allows for clean composition of different iteration strategies.
pub struct EngineIterator<E>
where
    E: Engine,
{
    pub(crate) engine: E,
}

/// Implementation of `Iterator` for [EngineIterator].
///
/// Each call to `next()` advances the engine by one generation and returns
/// the resulting generation data. The iterator continues indefinitely until
/// external termination conditions are applied - so always provide termination conditions.
impl<E> Iterator for EngineIterator<E>
where
    E: Engine,
{
    type Item = E::Epoch;

    fn next(&mut self) -> Option<Self::Item> {
        match self.engine.next() {
            Ok(epoch) => Some(epoch),
            Err(e) => panic!("{e}"),
        }
    }
}

/// Blanket implementation of the extension trait for any iterator over generations.
///
/// This implementation provides the extension methods to any iterator that yields
/// [`Generation<C, T>`] items, making the termination and monitoring functionality
/// available to a wide range of iteration patterns.
impl<I, C, T> EngineIteratorExt<C, T> for I
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
    T: Clone,
{
}

/// Extension trait providing advanced iteration capabilities for [Engine]s.
///
/// [`EngineIteratorExt`] adds powerful methods for controlling and monitoring the
/// evolutionary process, including various termination conditions, convergence
/// detection, stagnation monitoring, and logging capabilities.
///
/// # Generic Parameters
///
/// - `C`: The [Chromosome] type used by the engine
/// - `T`: The phenotype type produced by the engine
///
/// # Examples
///
/// ## Basic Termination
///
/// ```rust,ignore
/// use radiate_engines::*;
///
/// let engine = GeneticEngine::builder()
///     .codec(FloatCodec::vector(5, 0.0..1.0))
///     .fitness_fn(|x: Vec<f32>| x.iter().sum())
///     .build();
///
/// // Run for exactly 100 generations - this is essentially
/// // the same as using a '.Take(100)' on a traditional iterator & is
/// // also completely viable as a solution.
/// for generation in engine.iter().limit(Limit::Generation(100)) {
///     // Process each generation
/// }
///
/// // Run until fitness reaches 0.95
/// for generation in engine.iter().until_score(0.95) {
///     // Process until target fitness
/// }
///
/// // Run for 5 minutes
/// for generation in engine.iter().until_seconds(300) {
///     // Process for time limit
/// }
///
/// let last_generation = engine
///     .iter()
///     .logging() // Enable logging
///     .until_seconds(5)
///     .last(); // Get the last generation after 5 seconds
/// ```
///
/// ## Advanced Termination
///
/// ```rust,ignore
/// // Run until convergence (no improvement for 50 generations)
/// for generation in engine.iter().until_converged(50, 0.001) {
///     // Process until convergence
/// }
///
/// // Run until stagnation (no significant improvement for 100 generations)
/// for generation in engine.iter().until_stagnant(100, 0.01) {
///     // Process until stagnation
/// }
///
/// // Combine multiple limits
/// let combined_limit = Limit::Combined(vec![
///     Limit::Generation(1000),
///     Limit::Score(0.99),
///     Limit::Seconds(600.0),
/// ]);
///
/// for generation in engine.iter().limit(combined_limit) {
///     // Process with combined limits
/// }
/// ```
///
/// ## Monitoring and Logging
///
/// ```rust,ignore
/// // Add logging to see progress
/// for generation in engine.iter()
///     .take(100)
///     .logging() {
///     // Each generation will be logged automatically
/// }
/// ```
///
/// # Termination Strategies
///
/// The extension provides several termination strategies:
///
/// - **Generation Limits**: Stop after a fixed number of generations
/// - **Time Limits**: Stop after a specified duration
/// - **Score Thresholds**: Stop when fitness reaches a target value
/// - **Convergence Detection**: Stop when improvement rate falls below threshold
/// - **Stagnation Detection**: Stop when no significant improvement occurs
/// - **Combined Limits**: Apply multiple termination conditions
///
/// # Performance Considerations
///
/// - **Early Termination**: Conditions are checked efficiently without blocking
/// - **Memory Management**: History-based iterators use bounded memory
/// - **Composable Design**: Multiple limits can be combined efficiently
pub trait EngineIteratorExt<C, T>: Iterator<Item = Generation<C, T>>
where
    C: Chromosome,
    T: Clone,
{
    /// Limits iteration to a specified number of seconds.
    ///
    /// This method creates an iterator that stops when the cumulative execution
    /// time reaches the specified limit. The time is measured by the internal metric system,
    /// so it may not correspond exactly to wall-clock time. Instead, it is equal
    /// to the amount of actual compute time the engine takes.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum execution time in seconds as a floating-point value
    ///
    /// # Returns
    ///
    /// An iterator that respects the time limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Run for exactly 2.5 minutes
    /// for generation in engine.iter().until_seconds(150.0) {
    ///     // Process for 2.5 minutes
    /// }
    ///
    /// // Run for 30 seconds
    /// for generation in engine.iter().until_seconds(30.0) {
    ///     // Process for 30 seconds
    /// }
    /// ```
    fn until_seconds(self, limit: f64) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        DurationIterator {
            iter: self,
            limit: Duration::from_secs_f64(limit),
            done: false,
        }
    }

    /// Limits iteration to a specified duration.
    ///
    /// This method provides a more flexible way to specify time limits using
    /// Rust's `Duration` type, which can be constructed from various time units.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum execution time as a `Duration`
    ///
    /// # Returns
    ///
    /// An iterator that respects the duration limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::time::Duration;
    ///
    /// // Run for 5 minutes
    /// for generation in engine.iter().until_duration(Duration::from_secs(300)) {
    ///     // Process for 5 minutes
    /// }
    ///
    /// // Run for 1 hour and 30 minutes
    /// let duration = Duration::from_secs(3600) + Duration::from_secs(1800);
    /// for generation in engine.iter().until_duration(duration) {
    ///     // Process for 1.5 hours
    /// }
    ///
    /// // Run for 500 milliseconds
    /// for generation in engine.iter().until_duration(Duration::from_millis(500)) {
    ///     // Process for 500ms
    /// }
    /// ```
    fn until_duration(self, limit: impl Into<Duration>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        DurationIterator {
            iter: self,
            limit: limit.into(),
            done: false,
        }
    }

    /// Limits iteration until a target fitness score is reached.
    ///
    /// This method creates an iterator that stops when the best individual's
    /// fitness reaches or exceeds the specified threshold. The comparison
    /// respects the optimization objective (minimize/maximize) and works
    /// with both single and multi-objective problems.
    ///
    /// # Arguments
    ///
    /// * `limit` - The target fitness score to reach
    ///
    /// # Returns
    ///
    /// An iterator that stops when the target score is reached
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // For minimization problems, stop when score <= 0.01
    /// for generation in engine.iter().until_score(0.01) {
    ///     // Process until score drops below 0.01
    /// }
    ///
    /// // For maximization problems, stop when score >= 0.95
    /// for generation in engine.iter().until_score(0.95) {
    ///     // Process until score exceeds 0.95
    /// }
    ///
    /// // Multi-objective problems work automatically
    /// let target_scores = vec![0.01, 0.02, 0.03];
    /// for generation in engine.iter().until_score(target_scores) {
    ///     // Process until all objectives meet their targets
    /// }
    /// ```
    ///
    /// # Objective Handling
    ///
    /// The method automatically handles different optimization objectives:
    /// - **Minimize**: Stops when score <= target
    /// - **Maximize**: Stops when score >= target
    /// - **Multi-objective**: Stops when all objectives meet their targets
    fn until_score(self, limit: impl Into<Score>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        ScoreIterator {
            iter: self,
            limit: limit.into(),
            done: false,
        }
    }

    /// Limits iteration until convergence is detected.
    ///
    /// This method creates an iterator that stops when the improvement rate
    /// over a specified window of generations falls below a threshold. This
    /// is great for detecting when the algorithm has converged to a local
    /// or global optimum.
    ///
    /// # Arguments
    ///
    /// * `window` - Number of generations to consider for convergence detection
    /// * `epsilon` - Minimum improvement threshold (non-negative)
    ///
    /// # Returns
    ///
    /// An iterator that stops when convergence is detected
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Stop when no improvement > 0.001 over 50 generations
    /// for generation in engine.iter().until_converged(50, 0.001) {
    ///     // Process until convergence
    /// }
    ///
    /// // More sensitive convergence detection
    /// for generation in engine.iter().until_converged(20, 0.0001) {
    ///     // Process until convergence with smaller window and threshold
    /// }
    /// ```
    ///
    /// # Algorithm
    ///
    /// Convergence detection works by:
    /// 1. Maintaining a sliding window of fitness scores
    /// 2. Comparing the first and last scores in the window
    /// 3. Stopping when the difference is less than epsilon
    /// 4. The window size determines sensitivity to local fluctuations
    ///
    /// # Use Cases
    ///
    /// - **Local Optima**: Detect when stuck in local optima
    /// - **Global Convergence**: Identify when approaching global solution
    /// - **Resource Management**: Stop when further improvement is unlikely
    fn until_converged(self, window: usize, epsilon: f32) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        assert!(window > 0, "Window size must be greater than 0");
        assert!(epsilon >= 0.0, "Epsilon must be non-negative");

        ConvergenceIterator {
            iter: self,
            history: VecDeque::new(),
            window,
            epsilon,
            done: false,
        }
    }

    /// Limits iteration until stagnation is detected.
    ///
    /// This method creates an iterator that stops when no significant
    /// improvement occurs for a specified number of generations. This is
    /// great for detecting when the algorithm has plateaued and further
    /// progress is unlikely without parameter adjustments.
    ///
    /// # Arguments
    ///
    /// * `patience` - Number of generations to wait for improvement
    /// * `min_improvement` - Minimum improvement threshold (non-negative)
    ///
    /// # Returns
    ///
    /// An iterator that stops when stagnation is detected
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Stop when no improvement > 0.01 for 100 generations
    /// for generation in engine.iter().until_stagnant(100, 0.01) {
    ///     // Process until stagnation
    /// }
    ///
    /// // More sensitive stagnation detection
    /// for generation in engine.iter().until_stagnant(50, 0.001) {
    ///     // Process until stagnation with higher sensitivity
    /// }
    /// ```
    ///
    /// # Algorithm
    ///
    /// Stagnation detection works by:
    /// 1. Tracking the best fitness score seen so far
    /// 2. Counting generations since the last significant improvement
    /// 3. Stopping when the patience threshold is exceeded
    /// 4. Resetting the counter when improvement occurs
    ///
    /// # Use Cases
    ///
    /// - **Plateau Detection**: Identify when algorithm stops improving
    /// - **Parameter Tuning**: Signal when to adjust mutation rates
    /// - **Resource Management**: Stop when further computation is unlikely to help
    fn until_stagnant(
        self,
        patience: usize,
        min_improvement: f32,
    ) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        assert!(patience > 0, "Patience must be greater than 0");
        assert!(
            min_improvement >= 0.0,
            "Min improvement must be non-negative"
        );

        StagnationIterator {
            iter: self,
            best_score: None,
            patience,
            min_improvement,
            stagnant_count: 0,
            done: false,
        }
    }

    /// Applies a [Limit] specification to the iterator.
    ///
    /// This method provides a unified interface for applying various types
    /// of limits to the iteration process. It supports all [Limit] types and
    /// can combine multiple limits for complex termination conditions. This will
    /// stop evolution when any [Limit] is reached.
    ///
    /// # Arguments
    ///
    /// * `limit` - The limit specification to apply
    ///
    /// # Returns
    ///
    /// A boxed iterator that respects the specified limits
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use radiate_engines::Limit;
    ///
    /// // Single limit
    /// let limit = Limit::Generation(100);
    /// for generation in engine.iter().limit(limit) {
    ///     // Process for 100 generations
    /// }
    ///
    /// // Combined limits
    /// let combined = Limit::Combined(vec![
    ///     Limit::Generation(1000),
    ///     Limit::Score(0.99),
    ///     Limit::Seconds(300.0),
    /// ]);
    ///
    /// for generation in engine.iter().limit(combined) {
    ///     // Process until any limit is reached
    /// }
    /// ```
    ///
    /// # Limit Types
    ///
    /// The method supports all [Limit] types:
    /// - **Generation**: Stop after N generations
    /// - **Seconds**: Stop after N seconds
    /// - **Score**: Stop when target fitness is reached
    /// - **Convergence**: Stop when convergence is detected
    /// - **Combined**: Apply multiple limits simultaneously
    ///
    /// # Performance
    ///
    /// The method returns a boxed iterator to support dynamic dispatch of
    /// different limit types. This provides flexibility at a small runtime cost.
    fn limit(self, limit: impl Into<Limit>) -> Box<dyn Iterator<Item = Generation<C, T>>>
    where
        Self: Sized + 'static,
        C: 'static,
        T: 'static,
    {
        let limit = limit.into();

        match limit {
            Limit::Generation(lim) => Box::new(GenerationIterator {
                iter: self,
                max_index: lim,
                done: false,
            }),
            Limit::Seconds(sec) => Box::new(DurationIterator {
                iter: self,
                limit: sec,
                done: false,
            }),
            Limit::Score(score) => Box::new(ScoreIterator {
                iter: self,
                limit: score,
                done: false,
            }),
            Limit::Convergence(window, epsilon) => Box::new(ConvergenceIterator {
                iter: self,
                window,
                epsilon,
                done: false,
                history: VecDeque::new(),
            }),
            Limit::Combined(limits) => {
                let mut iter: Box<dyn Iterator<Item = Generation<C, T>>> = Box::new(self);
                for limit in limits {
                    iter = match limit {
                        Limit::Generation(lim) => Box::new(GenerationIterator {
                            iter,
                            max_index: lim,
                            done: false,
                        }),
                        Limit::Seconds(sec) => Box::new(DurationIterator {
                            iter,
                            limit: sec,
                            done: false,
                        }),
                        Limit::Score(score) => Box::new(ScoreIterator {
                            iter,
                            limit: score,
                            done: false,
                        }),
                        Limit::Convergence(window, epsilon) => {
                            Box::new(iter.until_converged(window, epsilon))
                        }
                        _ => iter,
                    };
                }

                iter
            }
        }
    }

    /// Adds logging to the iteration process.
    ///
    /// This method wraps the iterator with logging capabilities, automatically
    /// logging information about each generation including index, scores, and
    /// execution time.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Add logging to see progress
    /// for generation in engine.iter()
    ///     .limit(Limit::Generation(100))
    ///     .logging() {
    ///     // Each generation will be logged automatically
    ///     // Output format: "Epoch 1    | Score: 0.8500 | Time: 0.15s"
    /// }
    /// ```
    ///
    /// # Logging Format
    ///
    /// The logging provides structured information:
    /// - **Single Objective**: "Epoch N | Score: X.XXXX | Time: Y.YYs"
    /// - **Multi Objective**: "Epoch N | Scores: [X, Y, Z] | Time: Y.YYs"
    ///
    /// # Integration
    ///
    /// Logging integrates with the `tracing` crate, allowing you to:
    fn logging(self) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        init_logging();
        LoggingIterator { iter: self }
    }
}

/// Iterator that adds logging to each generation.
///
/// This iterator automatically logs information about each generation as it
/// is produced, providing real-time monitoring of the evolutionary process.
/// The logging format adapts to single and multi-objective problems.
struct LoggingIterator<I, C, T>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
}

/// Implementation of `Iterator` for [LoggingIterator].
///
/// Each call to `next()` retrieves the next generation, logs relevant
/// information, and returns the generation unchanged. The logging provides
/// structured output for monitoring and debugging.
impl<I, C, T> Iterator for LoggingIterator<I, C, T>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    type Item = Generation<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;

        match next.objective() {
            Objective::Single(_) => {
                info!(
                    "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
                    next.index(),
                    next.score().as_f32(),
                    next.time()
                );
            }
            Objective::Multi(_) => {
                info!(
                    "Epoch {:<4} | Scores: {:?} | Time: {:>5.2?}",
                    next.index(),
                    next.score(),
                    next.time()
                );
            }
        }

        Some(next)
    }
}

/// Iterator that limits iteration to a maximum number of generations.
///
/// This iterator stops producing items after the specified number of generations
/// has been reached. It's useful for controlling the computational budget
/// and ensuring the algorithm doesn't run indefinitely.
struct GenerationIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
    max_index: usize,
    done: bool,
}

/// Implementation of `Iterator` for [GenerationIterator].
///
/// The iterator produces generations until the maximum index is reached,
/// then stops producing items. Its really just a simple way to limit
/// computational effort.
impl<I, C, T> Iterator for GenerationIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    type Item = Generation<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.max_index == 0 || self.done {
            return None;
        }

        let next_ctx = self.iter.next()?;
        if next_ctx.index() >= self.max_index {
            self.done = true;
        }

        Some(next_ctx)
    }
}

/// Iterator that limits iteration to a maximum execution time.
///
/// This iterator stops producing items when the cumulative execution time
/// reaches the specified limit.
struct DurationIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
    limit: Duration,
    done: bool,
}

/// Implementation of `Iterator` for [DurationIterator].
///
/// The iterator produces generations until the time limit is reached,
/// then stops producing items. Time is measured from the [Engine]'s internal
/// metric system, which provides a consistent and accurate way to track
/// elapsed time.
impl<I, C, T> Iterator for DurationIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    type Item = Generation<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.limit <= Duration::from_millis(0) || self.done {
            return None;
        }

        let next = self.iter.next()?;
        if next.time() >= self.limit {
            self.done = true;
        }

        Some(next)
    }
}

/// Iterator that limits iteration until a target fitness score is reached.
///
/// This iterator stops producing items when the best individual's fitness
/// reaches or exceeds the specified threshold. The comparison respects
/// the optimization objective and works with multi-objective problems.
struct ScoreIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
    limit: Score,
    done: bool,
}

/// Implementation of `Iterator` for [ScoreIterator].
///
/// The iterator produces generations until the target fitness is reached,
/// respecting the optimization objective. For multi-objective problems,
/// all objectives must meet their targets.
impl<I, C, T> Iterator for ScoreIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    type Item = Generation<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let ctx = self.iter.next()?;

        let passed = match ctx.objective() {
            Objective::Single(obj) => match obj {
                Optimize::Minimize => ctx.score() > &self.limit,
                Optimize::Maximize => ctx.score() < &self.limit,
            },
            Objective::Multi(objs) => {
                let mut all_pass = true;
                for (i, score) in ctx.score().iter().enumerate() {
                    let passed = match objs[i] {
                        Optimize::Minimize => score > &self.limit[i],
                        Optimize::Maximize => score < &self.limit[i],
                    };

                    if !passed {
                        all_pass = false;
                        break;
                    }
                }

                all_pass
            }
        };

        if !passed {
            self.done = true;
        }

        Some(ctx)
    }
}

/// Iterator that limits iteration until convergence is detected.
///
/// This iterator stops producing items when the improvement rate over a
/// specified window of generations falls below a threshold.
struct ConvergenceIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
    history: VecDeque<f32>,
    window: usize,
    epsilon: f32,
    done: bool,
}

/// Implementation of `Iterator` for [ConvergenceIterator].
///
/// The iterator maintains a sliding window of fitness scores and stops
/// when the improvement rate falls below the epsilon threshold. This
/// provides adaptive termination based on algorithm behavior.
impl<I, C, T> Iterator for ConvergenceIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    type Item = Generation<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next_ctx = self.iter.next()?;
        let score = next_ctx.score().as_f32();

        self.history.push_back(score);
        if self.history.len() > self.window {
            self.history.pop_front();
        }

        if self.history.len() == self.window {
            let first = self.history.front().unwrap();
            let last = self.history.back().unwrap();
            if (first - last).abs() < self.epsilon {
                self.done = true;
            }
        }

        Some(next_ctx)
    }
}

/// Iterator that limits iteration until stagnation is detected.
///
/// This iterator stops producing items when no significant improvement
/// occurs for a specified number of generations.
struct StagnationIterator<I> {
    iter: I,
    best_score: Option<f32>,
    patience: usize,
    min_improvement: f32,
    stagnant_count: usize,
    done: bool,
}

/// Implementation of `Iterator` for [StagnationIterator].
///
/// The iterator tracks the best fitness score and stops when no
/// significant improvement occurs for the specified patience period.
/// This provides early termination for stuck algorithms.
impl<I, C, T> Iterator for StagnationIterator<I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    type Item = Generation<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let generation = self.iter.next()?;
        let current_score = generation.score().as_f32();

        match self.best_score {
            Some(best) => {
                if current_score - best > self.min_improvement {
                    self.best_score = Some(current_score);
                    self.stagnant_count = 0;
                } else {
                    self.stagnant_count += 1;
                    if self.stagnant_count >= self.patience {
                        self.done = true;
                    }
                }
            }
            None => {
                self.best_score = Some(current_score);
            }
        }

        Some(generation)
    }
}
