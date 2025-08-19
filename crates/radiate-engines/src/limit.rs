//! # Limit System
//!
//! This module provides a flexible and extensible limit system for controlling
//! the execution of genetic algorithms and evolutionary computations through the
//! EngineIterator. The `Limit` enum defines various types of
//! termination conditions that can be applied individually or combined for
//! complex control scenarios.
//!
//! The limit system supports multiple termination strategies:
//! - **Generation Limits**: Stop after a fixed number of generations
//! - **Time Limits**: Stop after a specified duration
//! - **Score Thresholds**: Stop when fitness targets are reached
//! - **Convergence Detection**: Stop when improvement rate falls below threshold
//! - **Combined Limits**: Apply multiple limits simultaneously

use radiate_core::Score;
use std::time::Duration;

/// Defines various types of limits for controlling genetic algorithm execution.
///
/// The `Limit` enum provides a unified interface for specifying when and how
/// evolutionary algorithms should terminate. Limits can be used individually
/// or combined to create complex termination scenarios that balance multiple
/// objectives like computation time, solution quality, and convergence.
///
/// # Limit Types
///
/// ## Generation Limits
/// Stop execution after a fixed number of generations, useful for controlling
/// computational budget and ensuring reproducible results.
///
/// ## Time Limits
/// Stop execution after a specified duration, useful for real-time applications
/// or when running on shared computing resources with time constraints.
///
/// ## Score Thresholds
/// Stop execution when fitness targets are reached, useful for problems where
/// you know the desired solution quality or have specific performance requirements.
///
/// ## Convergence Detection
/// Stop execution when the improvement rate falls below a threshold, useful
/// for detecting when the algorithm has converged to a local or global optimum.
///
/// ## Combined Limits
/// Apply multiple limits simultaneously, stopping when any limit is reached.
/// This provides flexible control for complex scenarios.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use radiate_engines::Limit;
/// use radiate_core::Score;
/// use std::time::Duration;
///
/// // Generation limit
/// let gen_limit = Limit::Generation(1000);
///
/// // Time limit
/// let time_limit = Limit::Seconds(Duration::from_secs(300));
///
/// // Score threshold
/// let score_limit = Limit::Score(Score::from(0.95));
///
/// // Convergence detection
/// let conv_limit = Limit::Convergence(50, 0.001);
/// ```
///
/// ## Combined Limits
///
/// ```rust
/// use radiate_engines::Limit;
/// use radiate_core::Score;
/// use std::time::Duration;
///
/// // Combine multiple limits
/// let combined = Limit::Combined(vec![
///     Limit::Generation(1000),           // Max 1000 generations
///     Limit::Seconds(Duration::from_secs(600)), // Max 10 minutes
///     Limit::Score(Score::from(0.99)),   // Stop at .99 fitness
/// ]);
///
/// // This will stop when ANY of the limits is reached
/// ```
///
/// ## Automatic Conversion
///
/// ```rust
/// use radiate_engines::Limit;
/// use std::time::Duration;
///
/// // Automatic conversion from common types
/// let gen_limit: Limit = 500.into();                    // Generation limit
/// let time_limit: Limit = Duration::from_secs(120).into(); // Time limit
/// let score_limit: Limit = 0.85f32.into();             // Score limit
/// let multi_score: Limit = vec![0.9, 0.8, 0.7].into(); // Multi-objective
/// let conv_limit: Limit = (25, 0.01f32).into();        // Convergence
/// ```
#[derive(Debug, Clone)]
pub enum Limit {
    Generation(usize),
    Seconds(Duration),
    Score(Score),
    Convergence(usize, f32),
    Combined(Vec<Limit>),
}

impl Into<Limit> for usize {
    fn into(self) -> Limit {
        Limit::Generation(self)
    }
}

impl Into<Limit> for Duration {
    fn into(self) -> Limit {
        Limit::Seconds(self)
    }
}

impl Into<Limit> for f32 {
    fn into(self) -> Limit {
        Limit::Score(Score::from(self))
    }
}

impl Into<Limit> for Vec<f32> {
    fn into(self) -> Limit {
        Limit::Score(Score::from(self))
    }
}

impl Into<Limit> for (usize, f32) {
    fn into(self) -> Limit {
        Limit::Convergence(self.0, self.1)
    }
}

impl Into<Limit> for Vec<Limit> {
    fn into(self) -> Limit {
        Limit::Combined(self)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_limit_conversions() {
        use super::Limit;
        use std::time::Duration;

        let gen_limit: Limit = 100.into();
        match gen_limit {
            Limit::Generation(n) => assert_eq!(n, 100),
            _ => panic!("Expected Generation limit"),
        }

        let time_limit: Limit = Duration::from_secs(60).into();
        match time_limit {
            Limit::Seconds(dur) => assert_eq!(dur, Duration::from_secs(60)),
            _ => panic!("Expected Seconds limit"),
        }

        let score_limit: Limit = 95.5f32.into();
        match score_limit {
            Limit::Score(score) => assert_eq!(score.as_f32(), 95.5),
            _ => panic!("Expected Score limit"),
        }

        let multi_score_limit: Limit = vec![90.0f32, 85.5f32, 78.0f32].into();
        match multi_score_limit {
            Limit::Score(score) => {
                assert_eq!(score[0], 90.0);
                assert_eq!(score[1], 85.5);
                assert_eq!(score[2], 78.0);
            }
            _ => panic!("Expected Multi Score limit"),
        }

        let conv_limit: Limit = (10, 0.01f32).into();
        match conv_limit {
            Limit::Convergence(gens, thresh) => {
                assert_eq!(gens, 10);
                assert_eq!(thresh, 0.01);
            }
            _ => panic!("Expected Convergence limit"),
        }

        let generation_combined_limit: Limit = 100.into();
        let duration_combined_limit: Limit = Duration::from_secs(30).into();
        let combined_limit: Limit = vec![generation_combined_limit, duration_combined_limit].into();
        match combined_limit {
            Limit::Combined(limits) => assert_eq!(limits.len(), 2),
            _ => panic!("Expected Combined limit"),
        }
    }
}
