use crate::{Generation, GeneticEngine};
use radiate_core::{Chromosome, Engine, Epoch, Score};
use std::{collections::VecDeque, time::Duration};

pub struct EngineIterator<C, T>
where
    C: Chromosome,
{
    pub(crate) engine: GeneticEngine<C, T>,
}

impl<C, T> Iterator for EngineIterator<C, T>
where
    C: Chromosome,
    T: Clone,
{
    type Item = Generation<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.engine.next())
    }
}

impl<C, T> EngineIteratorExt<C, T> for EngineIterator<C, T>
where
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
        self.take_while(move |ctx| ctx.seconds() < limit)
    }

    fn until_duration(self, limit: impl Into<Duration>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let limit = limit.into();
        self.take_while(move |ctx| ctx.time() < limit)
    }

    fn until_score_above(self, limit: impl Into<Score>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let limit = limit.into();
        self.take_while(move |ctx| ctx.score() < &limit)
    }

    fn until_score_below(self, limit: impl Into<Score>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let limit = limit.into();
        self.take_while(move |ctx| ctx.score() > &limit)
    }

    fn until_score_equal(self, limit: impl Into<Score>) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        let limit = limit.into();
        self.take_while(move |ctx| ctx.score() == &limit)
    }

    fn until_converged(self, window: usize, epsilon: f32) -> impl Iterator<Item = Generation<C, T>>
    where
        Self: Sized,
    {
        ConverganceIterator {
            iter: self,
            history: VecDeque::new(),
            window,
            epsilon,
            done: false,
        }
    }
}

struct ConverganceIterator<I, C, T>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
    T: Clone,
{
    iter: I,
    history: VecDeque<f32>,
    window: usize,
    epsilon: f32,
    done: bool,
}

impl<I, C, T> Iterator for ConverganceIterator<I, C, T>
where
    I: Iterator<Item = Generation<C, T>>,
    C: Chromosome,
    T: Clone,
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
