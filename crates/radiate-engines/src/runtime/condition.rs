use crate::{context::RuntimeContext, runtime::iter::EngineGuard};
use radiate_core::{AnyValue, Engine, Evaluate, Expr, Metric, Objective, Optimize, Score};
use std::{collections::VecDeque, sync::Arc, time::Duration};

pub trait RuntimeLimit<E>
where
    E: Engine,
{
    fn check<'a>(&mut self, snapshot: &EngineGuard<'a, E>) -> bool;
}

pub(crate) struct GenerationLimit {
    limit: usize,
}

impl<E> RuntimeLimit<E> for GenerationLimit
where
    E: Engine,
    E::Context: RuntimeContext,
{
    fn check<'a>(&mut self, snapshot: &EngineGuard<'a, E>) -> bool {
        snapshot.view().index() < self.limit
    }
}

impl From<usize> for GenerationLimit {
    fn from(value: usize) -> Self {
        GenerationLimit { limit: value }
    }
}

pub(crate) struct DurationLimit {
    limit: Duration,
}

impl<E> RuntimeLimit<E> for DurationLimit
where
    E: Engine,
    E::Context: RuntimeContext,
{
    fn check<'a>(&mut self, snapshot: &EngineGuard<'a, E>) -> bool {
        let time = snapshot.view().time();
        time < self.limit
    }
}

impl<T> From<T> for DurationLimit
where
    T: Into<Duration>,
{
    fn from(value: T) -> Self {
        DurationLimit {
            limit: value.into(),
        }
    }
}

pub(crate) struct ScoreLimit {
    limit: Score,
}

impl<E> RuntimeLimit<E> for ScoreLimit
where
    E: Engine,
    E::Context: RuntimeContext,
{
    fn check<'a>(&mut self, guard: &EngineGuard<'a, E>) -> bool {
        let snapshot = guard.view();
        let Some(score) = snapshot.score() else {
            return true; // No score yet, so we can't say we've failed the limit
        };

        let passed = match snapshot.objective() {
            Objective::Single(obj) => match obj {
                Optimize::Minimize => score > &self.limit,
                Optimize::Maximize => score < &self.limit,
            },
            Objective::Multi(objs) => {
                let mut all_pass = true;
                for (i, score) in score.iter().enumerate() {
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

        // if !passed {
        //     self.done = true;
        // }

        passed
    }
}

impl<T> From<T> for ScoreLimit
where
    T: Into<Score>,
{
    fn from(value: T) -> Self {
        ScoreLimit {
            limit: value.into(),
        }
    }
}

#[allow(dead_code)]
pub(crate) struct ConvergenceLimit {
    window: usize,
    epsilon: f32,
    buffer: VecDeque<Score>,
}

impl ConvergenceLimit {
    pub fn new(window: usize, epsilon: f32) -> Self {
        ConvergenceLimit {
            window,
            epsilon,
            buffer: VecDeque::with_capacity(window),
        }
    }
}

impl<E> RuntimeLimit<E> for ConvergenceLimit
where
    E: Engine,
    E::Context: RuntimeContext,
{
    fn check<'a>(&mut self, guard: &EngineGuard<'a, E>) -> bool {
        let snapshot = guard.view();
        let Some(score) = snapshot.score() else {
            return true; // No score yet, so we can't say we've failed the limit
        };

        self.buffer.push_back(score.clone());
        if self.buffer.len() > self.window {
            self.buffer.pop_front();
        }

        if self.buffer.len() < self.window {
            return true;
        }

        let first = &self.buffer[0];
        let last = &self.buffer[self.buffer.len() - 1];

        let improved = match snapshot.objective() {
            Objective::Single(obj) => match obj {
                Optimize::Minimize => first.as_f32() - last.as_f32(),
                Optimize::Maximize => last.as_f32() - first.as_f32(),
            },
            Objective::Multi(objs) => {
                let mut acc = 0.0;
                for (i, obj) in objs.iter().enumerate() {
                    let improvement = match obj {
                        Optimize::Minimize => first[i] - last[i],
                        Optimize::Maximize => last[i] - first[i],
                    };

                    acc += improvement;
                }
                acc
            }
        };

        improved > self.epsilon
    }
}

pub(crate) struct ExprLimit {
    limit: Expr,
}

impl From<Expr> for ExprLimit {
    fn from(value: Expr) -> Self {
        ExprLimit { limit: value }
    }
}

impl<E> RuntimeLimit<E> for ExprLimit
where
    E: Engine,
    E::Context: RuntimeContext,
{
    fn check<'a>(&mut self, guard: &EngineGuard<'a, E>) -> bool {
        let snapshot = guard.view();
        let metrics = snapshot.metrics();
        let result = self.limit.eval(metrics).unwrap_or(AnyValue::Null);

        if let AnyValue::Bool(b) = result {
            !b
        } else {
            panic!(
                "Expression did not evaluate to a boolean value: {:?}",
                result
            );
        }
    }
}

pub(crate) struct MetricLimit {
    name: String,
    condition: Arc<dyn Fn(&Metric) -> bool>,
}

impl MetricLimit {
    pub fn new(name: String, condition: Arc<dyn Fn(&Metric) -> bool>) -> Self {
        MetricLimit { name, condition }
    }
}

impl<E> RuntimeLimit<E> for MetricLimit
where
    E: Engine,
    E::Context: RuntimeContext,
{
    fn check<'a>(&mut self, guard: &EngineGuard<'a, E>) -> bool {
        let snapshot = guard.view();
        if let Some(metric) = snapshot.metrics().get(&self.name) {
            (self.condition)(metric)
        } else {
            true // If the metric doesn't exist, we can't say we've failed the limit
        }
    }
}
