use crate::{Generation, Limit};
use radiate_core::{Chromosome, Engine, Objective, Optimize, Score};
use std::{collections::VecDeque, time::Duration};

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
        SecondsIterator {
            iter: self,
            limit: Duration::from_secs_f64(limit),
            done: false,
        }
    }

    fn until_duration(self, limit: impl Into<Duration>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let limit = limit.into();
        SecondsIterator {
            iter: self,
            limit,
            done: false,
        }
    }

    fn until_score(self, limit: impl Into<Score>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let lim = limit.into();

        ScoreIterator {
            iter: self,
            limit: lim,
            done: false,
        }
    }

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

    fn limit(
        self,
        limit: impl Into<Limit>,
    ) -> LimitIterator<Box<dyn Iterator<Item = Generation<C, T>>>>
    where
        Self: Sized + 'static,
        C: 'static,
        T: 'static,
    {
        let limit = limit.into();

        match limit {
            Limit::Generation(lim) => LimitIterator {
                iter: Box::new(GenerationIterator {
                    iter: self,
                    max_index: lim,
                    done: false,
                }),
            },
            Limit::Seconds(sec) => LimitIterator {
                iter: Box::new(SecondsIterator {
                    iter: self,
                    limit: sec,
                    done: false,
                }),
            },
            Limit::Score(score) => LimitIterator {
                iter: Box::new(ScoreIterator {
                    iter: self,
                    limit: score,
                    done: false,
                }),
            },
            Limit::Convergence(window, epsilon) => LimitIterator {
                iter: Box::new(self.until_converged(window, epsilon)),
            },
            Limit::Combined(limits) => {
                let mut iter: Box<dyn Iterator<Item = Generation<C, T>>> = Box::new(self);
                for limit in limits {
                    iter = match limit {
                        Limit::Generation(lim) => Box::new(GenerationIterator {
                            iter,
                            max_index: lim,
                            done: false,
                        }),
                        Limit::Seconds(sec) => Box::new(SecondsIterator {
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

                LimitIterator {
                    iter: Box::new(iter),
                }
            }
        }
    }
}

struct GenerationIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
    max_index: usize,
    done: bool,
}

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

struct SecondsIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
    limit: Duration,
    done: bool,
}

impl<I, C, T> Iterator for SecondsIterator<C, T, I>
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

struct ScoreIterator<C, T, I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    iter: I,
    limit: Score,
    done: bool,
}

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

pub struct LimitIterator<I> {
    iter: I,
}

impl<I, C, T> Iterator for LimitIterator<I>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
