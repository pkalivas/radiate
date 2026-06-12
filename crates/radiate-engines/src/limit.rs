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

use crate::{EvolutionContext, Generation, generation::GenerationView, runtime::RuntimeLimit};
use radiate_core::{
    AnyValue, Chromosome, Engine, Evaluate, Expr, Metric, Objective, Optimize, Score,
    error::RadiateResult,
};
use radiate_error::radiate_bail;
use std::{collections::VecDeque, fmt::Debug, sync::Arc, time::Duration};

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
/// let gen_limit: Limit = 500.into();                          // Generation limit
/// let time_limit: Limit = Duration::from_secs(120).into();    // Time limit
/// let score_limit: Limit = 0.85f32.into();                    // Score limit
/// let multi_score: Limit = vec![0.9, 0.8, 0.7].into();        // Multi-objective
/// let conv_limit: Limit = (25, 0.01f32).into();               // Convergence
/// ```
#[derive(Clone)]
pub enum Limit {
    Generation(usize),
    Seconds(Duration),
    Score(Score),
    Convergence(usize, f32, VecDeque<f32>),
    Combined(Vec<Limit>),
    Metric(String, Arc<dyn Fn(&Metric) -> bool + Send + Sync>),
    Expr(Expr),
}

impl<C, T, E> RuntimeLimit<E> for Limit
where
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>>,
    C: Chromosome + Clone,
    T: Clone + Send + Sync,
{
    fn proceed(&mut self, ctx: &E::Ctx) -> RadiateResult<bool> {
        match self {
            Limit::Generation(gens) => Ok(ctx.index < *gens),
            Limit::Seconds(secs) => {
                let total_time = ctx
                    .metrics
                    .time()
                    .and_then(|m| m.times().map(|t| t.sum()))
                    .unwrap_or_default();
                Ok(total_time < *secs)
            }
            Limit::Score(limit) => {
                let Some(score) = &ctx.score else {
                    return Ok(true);
                };

                Ok(match &ctx.objective {
                    Objective::Single(obj) => match obj {
                        Optimize::Minimize => score > limit,
                        Optimize::Maximize => score < limit,
                    },
                    Objective::Multi(objs) => {
                        let mut all_pass = true;
                        for (i, score) in score.iter().enumerate() {
                            let passed = match objs[i] {
                                Optimize::Minimize => score > &limit[i],
                                Optimize::Maximize => score < &limit[i],
                            };

                            if !passed {
                                all_pass = false;
                                break;
                            }
                        }

                        all_pass
                    }
                })
            }
            Limit::Convergence(window, epsilon, history) => {
                let Some(current_score) = &ctx.score else {
                    return Ok(true);
                };

                history.push_back(current_score.as_f32());
                if history.len() > *window {
                    history.pop_front();
                }

                if history.len() < *window {
                    return Ok(true);
                }

                let first = history.front().unwrap();
                let last = history.back().unwrap();

                let improved = match &ctx.objective {
                    Objective::Single(_) => last - first,
                    Objective::Multi(_) => {
                        let mut total_improvement = 0.0;
                        for (i, score) in history.iter().enumerate() {
                            let improvement = match &ctx.objective {
                                Objective::Multi(objs) => match objs[i] {
                                    Optimize::Minimize => score - first,
                                    Optimize::Maximize => first - score,
                                },
                                _ => 0.0,
                            };
                            total_improvement += improvement;
                        }
                        total_improvement / history.len() as f32
                    }
                };

                Ok(improved.abs() > *epsilon)
            }
            Limit::Combined(limits) => limits
                .iter_mut()
                .map(|limit| <Limit as RuntimeLimit<E>>::proceed(limit, ctx))
                .collect::<RadiateResult<Vec<bool>>>()
                .map(|proceed| proceed.iter().all(|&p| p)),
            Limit::Metric(name, predicate) => Ok(if let Some(metric) = ctx.metrics.get(name) {
                predicate(metric)
            } else {
                true
            }),
            Limit::Expr(expr) => {
                let metrics = &ctx.metrics;
                let result = expr.eval(metrics).unwrap_or(AnyValue::Null);

                if let AnyValue::Bool(b) = result {
                    Ok(!b)
                } else {
                    radiate_bail!(Engine: format!(
                        "Expression did not evaluate to a boolean value: {:?}",
                        result
                    ))
                }
            }
        }
    }
}

impl From<usize> for Limit {
    fn from(value: usize) -> Self {
        Limit::Generation(value)
    }
}

impl From<Duration> for Limit {
    fn from(value: Duration) -> Self {
        Limit::Seconds(value)
    }
}

impl From<f32> for Limit {
    fn from(value: f32) -> Self {
        Limit::Score(Score::from(value))
    }
}

impl From<Vec<f32>> for Limit {
    fn from(value: Vec<f32>) -> Self {
        Limit::Score(Score::from(value))
    }
}

impl From<(usize, f32)> for Limit {
    fn from((window, epsilon): (usize, f32)) -> Self {
        Limit::Convergence(window, epsilon, VecDeque::with_capacity(window))
    }
}

impl From<Expr> for Limit {
    fn from(value: Expr) -> Self {
        Limit::Expr(value)
    }
}

impl<F> From<(&str, F)> for Limit
where
    F: Fn(&Metric) -> bool + Send + Sync + 'static,
{
    fn from(value: (&str, F)) -> Self {
        Limit::Metric(value.0.to_string(), Arc::new(value.1))
    }
}

impl From<Vec<Limit>> for Limit {
    fn from(value: Vec<Limit>) -> Self {
        Limit::Combined(value)
    }
}

impl From<(Limit, Limit)> for Limit {
    fn from(value: (Limit, Limit)) -> Self {
        Limit::Combined(vec![value.0, value.1])
    }
}

impl From<(Limit, Limit, Limit)> for Limit {
    fn from(value: (Limit, Limit, Limit)) -> Self {
        Limit::Combined(vec![value.0, value.1, value.2])
    }
}

impl From<(Limit, Limit, Limit, Limit)> for Limit {
    fn from(value: (Limit, Limit, Limit, Limit)) -> Self {
        Limit::Combined(vec![value.0, value.1, value.2, value.3])
    }
}

impl Debug for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Limit::Generation(gens) => write!(f, "Generation({gens})"),
            Limit::Seconds(secs) => write!(f, "Seconds({secs:?})"),
            Limit::Score(score) => write!(f, "Score({:?})", score.as_f32()),
            Limit::Convergence(window, epsilon, _) => {
                write!(f, "Convergence(window: {window}, epsilon: {epsilon})")
            }
            Limit::Combined(limits) => write!(f, "Combined({limits:?})"),
            Limit::Metric(name, _) => write!(f, "MetricLimit({name})"),
            Limit::Expr(expr) => write!(f, "ExprLimit({expr:?})"),
        }
    }
}

impl<C, T, E, F> RuntimeLimit<E> for F
where
    C: Chromosome,
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>>,
    F: Fn(GenerationView<C, T>) -> bool,
{
    fn proceed(&mut self, snapshot: &E::Ctx) -> RadiateResult<bool> {
        let view = GenerationView::new(snapshot);
        Ok(!(self)(view))
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
            Limit::Convergence(gens, thresh, _) => {
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
