use crate::{Generation, GeneticEngine};
use radiate_core::{Chromosome, Engine, Epoch, Score, engine::EngineContext, objectives::Scored};
use std::{collections::VecDeque, time::Duration};

pub struct EngineIterator<C, T, E>
where
    C: Chromosome,
    T: Clone,
    E: Epoch<C>,
{
    pub(crate) engine: GeneticEngine<C, T, E>,
}

impl<C, T, E> EngineIterator<C, T, E>
where
    C: Chromosome,
    T: Clone,
    E: Epoch<C>,
{
    pub fn new(engine: GeneticEngine<C, T, E>) -> Self {
        EngineIterator { engine }
    }
}

impl<C, T> EngineIterator<C, T, Generation<C, T>>
where
    C: Chromosome,
    T: Clone,
{
}

impl<C, T, E> Iterator for EngineIterator<C, T, E>
where
    C: Chromosome,
    T: Clone,
    E: Epoch<C> + for<'a> From<&'a EngineContext<C, T>>,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.engine.next())
    }
}

pub trait EngineIteratorExt<C, T, E>: Iterator<Item = E>
where
    C: Chromosome,
    T: Clone,
    E: Epoch<C>,
{
    fn iter(self) -> EngineIterator<C, T, E>;

    fn until_seconds(self, limit: f64) -> impl Iterator<Item = E>
    where
        Self: Sized,
    {
        self.take_while(move |ctx| ctx.seconds() < limit)
    }

    fn until_duration(self, limit: impl Into<Duration>) -> impl Iterator<Item = E>
    where
        Self: Sized,
    {
        let limit = limit.into();
        self.take_while(move |ctx| ctx.time() < limit)
    }

    fn until_score_above(self, limit: impl Into<Score>) -> impl Iterator<Item = E>
    where
        Self: Sized,
        E: Scored,
    {
        let limit = limit.into();
        self.take_while(move |ctx| ctx.score().map_or(true, |score| score < &limit))
    }

    fn until_score_below(self, limit: impl Into<Score>) -> impl Iterator<Item = E>
    where
        Self: Sized,
        E: Scored,
    {
        let limit = limit.into();
        self.take_while(move |ctx| ctx.score().map_or(true, |score| score > &limit))
    }

    fn until_converged(self, window: usize, epsilon: f32) -> impl Iterator<Item = E>
    where
        Self: Sized,
        E: Scored,
    {
        struct ConverganceIterator<I, E>
        where
            I: Iterator<Item = E>,
            E: Scored,
        {
            iter: I,
            history: VecDeque<f32>,
            window: usize,
            epsilon: f32,
            done: bool,
        }

        impl<I, E> Iterator for ConverganceIterator<I, E>
        where
            I: Iterator<Item = E>,
            E: Scored,
        {
            type Item = E;

            fn next(&mut self) -> Option<Self::Item> {
                if self.done {
                    return None;
                }

                let next_ctx = self.iter.next()?;
                let score = next_ctx.score()?.as_f32();

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

        ConverganceIterator {
            iter: self,
            history: VecDeque::new(),
            window,
            epsilon,
            done: false,
        }
    }
}

impl<C, T, E> EngineIteratorExt<C, T, E> for EngineIterator<C, T, E>
where
    C: Chromosome,
    T: Clone,
    E: Epoch<C> + for<'a> From<&'a EngineContext<C, T>>,
{
    fn iter(self) -> EngineIterator<C, T, E> {
        self
    }
}
