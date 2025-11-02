//! # Engine Traits
//!
//! This module provides the core engine abstraction for genetic algorithms and evolutionary
//! computation. The [Engine] trait defines the basic interface for evolutionary engines,
//! while `EngineExt` provides convenient extension methods for running engines with
//! custom termination conditions.
//!
//! The engine system is designed to be flexible and extensible, allowing different
//! evolutionary algorithms to implement their own epoch types and progression logic
//! while providing a common interface for execution control.

use radiate_error::Result;

/// A trait representing an evolutionary computation engine.
//
/// The [Engine] trait defines the fundamental interface for evolutionary algorithms.
/// Implementors define how the algorithm progresses from one generation/epoch to the
/// next, encapsulating the core evolutionary logic.
///
/// It is intentially esentially an iterator.
///
/// # Generic Parameters
///
/// - `Epoch`: The type representing a single step or generation in the evolutionary process
///
/// # Examples
///
/// ```rust
/// use radiate_core::engine::{Engine, EngineExt};
/// use radiate_error::RadiateError;
///
/// #[derive(Default)]
/// struct MyEngine {
///     generation: usize,
///     population: Vec<i32>,
/// }
///
/// #[derive(Debug, Clone)]
/// struct MyEpoch {
///     generation: usize,
///     population_size: usize,
/// }
///
/// impl Engine for MyEngine {
///     type Epoch = MyEpoch;
///     
///     fn next(&mut self) -> Result<Self::Epoch, RadiateError> {
///         // Perform one generation of evolution
///         // ... evolve population ...
///         self.generation += 1;
///         
///         Ok(MyEpoch {
///             generation: self.generation,
///             population_size: self.population.len()
///         })
///     }
/// }
///
/// // Use the engine with a termination condition
/// let mut engine = MyEngine::default();
/// let final_epoch = engine.run(|epoch| epoch.generation >= 10);
/// println!("Final generation: {}", final_epoch.generation);
/// ```
///
/// # Design Philosophy
///
/// The [Engine] trait is intentionally minimal, focusing on the core concept of
/// progression through evolutionary time. This allows for maximum flexibility in
/// implementing different evolutionary algorithms while maintaining a small, consistent
/// interface for execution control.
pub trait Engine {
    /// The type representing a single epoch or generation in the evolutionary process.
    ///
    /// The epoch type should contain all relevant information about the current
    /// state of the evolutionary algorithm, such as:
    /// - Generation number
    /// - Population statistics
    /// - Best fitness values
    /// - Convergence metrics
    /// - Any other state information needed for monitoring or decision making
    type Epoch;

    /// Advances the engine to the next epoch or generation.
    ///
    /// This method encapsulates one complete iteration of the evolutionary algorithm.
    /// It should perform all necessary operations to progress the population from
    /// the current state to the next generation, including:
    /// - Fitness evaluation
    /// - Selection
    /// - Reproduction (crossover and mutation)
    /// - Population replacement
    /// - Any other evolutionary operators
    ///
    /// # Returns
    ///
    /// An instance of `Self::Epoch` representing the new state after the evolution step
    ///
    /// # Side Effects
    ///
    /// This method is mutable for allowance of modification of the internal state of the engine,
    /// advancing the evolutionary process. The engine should maintain its state between calls
    /// to allow for continuous evolution over multiple generations.
    ///
    /// # Performance
    ///
    /// This method is called repeatedly during execution, so it should be
    /// optimized for performance.
    fn next(&mut self) -> Result<Self::Epoch>;
}

/// Extension trait providing convenient methods for running engines with custom logic.
///
/// `EngineExt` provides additional functionality for engines without requiring
/// changes to the core [Engine] trait. This follows the Rust pattern of using
/// extension traits to add functionality to existing types.
///
/// # Generic Parameters
///
/// - `E`: The engine type that this extension applies to
///
/// # Design Benefits
///
/// - **Separation of Concerns**: Core engine logic is separate from execution control
/// - **Flexibility**: Different termination conditions can be easily implemented
/// - **Reusability**: The same engine can be run with different stopping criteria
/// - **Testability**: Termination logic can be tested independently of engine logic
pub trait EngineExt<E: Engine> {
    /// Runs the engine until the specified termination condition is met.
    ///
    /// This method continuously calls `engine.next()` until the provided closure
    /// returns `true`, indicating that the termination condition has been satisfied.
    /// The final epoch is returned, allowing you to inspect the final state of
    /// the evolutionary process.
    ///
    /// # Arguments
    ///
    /// * `limit` - A closure that takes the current epoch and returns `true` when
    ///             the engine should stop, `false` to continue
    ///
    /// # Returns
    ///
    /// The epoch that satisfied the termination condition
    ///
    /// # Termination Conditions
    ///
    /// Common termination conditions include:
    /// - **Generation Limit**: Stop after a fixed number of generations
    /// - **Fitness Threshold**: Stop when best fitness reaches a target value
    /// - **Convergence**: Stop when population diversity or fitness improvement is minimal
    /// - **Time Limit**: Stop after a certain amount of computation time
    /// - **Solution Quality**: Stop when a satisfactory solution is found
    ///
    /// # Performance Considerations
    ///
    /// - The termination condition is checked after every epoch, so keep it lightweight
    /// - Avoid expensive computations in the termination closure
    /// - Consider using early termination for conditions that can be checked incrementally
    ///
    /// # Infinite Loops
    ///
    /// Be careful to ensure that your termination condition will eventually be met,
    /// especially when using complex logic. An infinite loop will cause the program
    /// to hang indefinitely.
    fn run<F>(&mut self, limit: F) -> E::Epoch
    where
        F: Fn(&E::Epoch) -> bool;
}

/// Blanket implementation of [EngineExt] for all types that implement [Engine].
///
/// This implementation provides the `run` method to any type that implements
/// the [Engine] trait, without requiring manual implementation.
///
/// # Implementation Details
///
/// The `run` method implements a simple loop that:
/// 1. Calls `self.next()` to advance the engine
/// 2. Checks the termination condition using the provided closure
/// 3. Breaks and returns the final epoch when the condition is met
impl<E> EngineExt<E> for E
where
    E: Engine,
{
    fn run<F>(&mut self, limit: F) -> E::Epoch
    where
        F: Fn(&E::Epoch) -> bool,
    {
        loop {
            match self.next() {
                Ok(epoch) => {
                    if limit(&epoch) {
                        return epoch;
                    }
                }
                Err(e) => {
                    panic!("{e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEpoch {
        generation: usize,
        fitness: f32,
    }

    #[derive(Default)]
    struct MockEngine {
        generation: usize,
    }

    impl Engine for MockEngine {
        type Epoch = MockEpoch;

        fn next(&mut self) -> Result<Self::Epoch> {
            self.generation += 1;
            Ok(MockEpoch {
                generation: self.generation,
                fitness: 1.0 / (self.generation as f32),
            })
        }
    }

    #[test]
    fn test_engine_next() {
        let mut engine = MockEngine::default();

        let epoch1 = engine.next().unwrap();
        assert_eq!(epoch1.generation, 1);
        assert_eq!(epoch1.fitness, 1.0);

        let epoch2 = engine.next().unwrap();
        assert_eq!(epoch2.generation, 2);
        assert_eq!(epoch2.fitness, 0.5);
    }

    #[test]
    fn test_engine_ext_run_generation_limit() {
        let mut engine = MockEngine::default();

        let final_epoch = engine.run(|epoch| epoch.generation >= 3);

        assert_eq!(final_epoch.generation, 3);
        assert_eq!(final_epoch.fitness, 1.0 / 3.0);
    }

    #[test]
    fn test_engine_ext_run_fitness_limit() {
        let mut engine = MockEngine::default();

        let final_epoch = engine.run(|epoch| epoch.fitness < 0.3);

        // Should stop when fitness drops below 0.3
        // 1/4 = 0.25, so it should stop at generation 4
        assert_eq!(final_epoch.generation, 4);
        assert_eq!(final_epoch.fitness, 0.25);
    }

    #[test]
    fn test_engine_ext_run_complex_condition() {
        let mut engine = MockEngine::default();

        let final_epoch = engine.run(|epoch| epoch.generation >= 5 || epoch.fitness < 0.2);

        // Should stop at generation 5 due to generation limit
        // (fitness at gen 5 is 0.2, which doesn't meet the fitness condition)
        assert_eq!(final_epoch.generation, 5);
        assert_eq!(final_epoch.fitness, 0.2);
    }

    #[test]
    fn test_engine_ext_run_immediate_termination() {
        let mut engine = MockEngine::default();

        let final_epoch = engine.run(|_| true);

        // Should stop immediately after first epoch
        assert_eq!(final_epoch.generation, 1);
        assert_eq!(final_epoch.fitness, 1.0);
    }

    #[test]
    fn test_engine_ext_run_zero_generations() {
        let mut engine = MockEngine::default();

        let final_epoch = engine.run(|epoch| epoch.generation > 0);

        // Should run at least one generation
        assert_eq!(final_epoch.generation, 1);
    }

    #[test]
    fn test_engine_ext_method_chaining() {
        let mut engine = MockEngine::default();

        // Test that we can call run multiple times on the same engine
        let epoch1 = engine.run(|epoch| epoch.generation >= 2);
        assert_eq!(epoch1.generation, 2);

        let epoch2 = engine.run(|epoch| epoch.generation >= 4);
        assert_eq!(epoch2.generation, 4);
    }
}
