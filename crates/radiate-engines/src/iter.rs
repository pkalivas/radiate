use radiate_core::{Chromosome, Engine, Objective, Optimize, Score};
use std::{collections::VecDeque, time::Duration};

use crate::Generation;

pub struct EngineIterator<E>
where
    E: Engine,
{
    pub(crate) engine: E,
}

impl<E> Iterator for EngineIterator<E>
where
    E: Engine,
{
    type Item = E::Epoch;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.engine.next())
    }
}

impl<I, C, T> EngineIteratorExt<C, T> for I
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
    T: Clone,
{
}

pub trait EngineIteratorExt<C, T>: Iterator<Item = Generation<C, T>>
where
    C: Chromosome,
    T: Clone,
{
    fn until_seconds(self, limit: f64) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        self.skip_while(move |ctx| ctx.seconds() < limit)
    }

    fn until_duration(self, limit: impl Into<Duration>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let limit = limit.into();
        self.skip_while(move |ctx| ctx.time() < limit)
    }

    fn until_score(self, limit: impl Into<Score>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let lim = limit.into();
        self.skip_while(move |ctx| match ctx.objective() {
            Objective::Single(obj) => match obj {
                Optimize::Minimize => ctx.score() > &lim,
                Optimize::Maximize => ctx.score() < &lim,
            },
            Objective::Multi(objs) => {
                let mut all_pass = true;
                for (i, score) in ctx.score().iter().enumerate() {
                    let passed = match objs[i] {
                        Optimize::Minimize => score > &lim[i],
                        Optimize::Maximize => score < &lim[i],
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

    fn until_converged(self, window: usize, epsilon: f32) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        assert!(window > 0, "Window size must be greater than 0");
        assert!(epsilon >= 0.0, "Epsilon must be non-negative");

        ConverganceIterator {
            iter: self,
            history: VecDeque::new(),
            window,
            epsilon,
            done: false,
        }
    }
}

struct ConverganceIterator<C, T, I>
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

impl<I, C, T> Iterator for ConverganceIterator<C, T, I>
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
