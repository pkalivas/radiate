use crate::GeneticEngine;
use radiate_core::{Chromosome, Engine, Epoch, Score, engine::Context, objectives::Scored};
use std::{collections::VecDeque, time::Duration};

pub struct EngineIterator<C, T, E>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
    E: Epoch,
{
    pub(crate) engine: GeneticEngine<C, T, E>,
}

impl<C, T, E> Iterator for EngineIterator<C, T, E>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
    E: Epoch<Chromosome = C> + for<'a> From<&'a Context<C, T>>,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.engine.next())
    }
}

impl<I, C, T, E> EngineIteratorExt<C, T, E> for I
where
    I: Iterator<Item = E>,
    C: Chromosome,
    T: Clone,
    E: Epoch<Chromosome = C> + for<'a> From<&'a Context<C, T>>,
{
}

pub trait EngineIteratorExt<C, T, E>: Iterator<Item = E>
where
    C: Chromosome,
    T: Clone,
    E: Epoch<Chromosome = C>,
{
    fn until_seconds(self, limit: f64) -> impl Iterator<Item = E>
    where
        Self: Sized,
    {
        self.skip_while(move |ctx| ctx.seconds() < limit)
    }

    fn until_duration(self, limit: impl Into<Duration>) -> impl Iterator<Item = E>
    where
        Self: Sized,
    {
        let limit = limit.into();
        self.skip_while(move |ctx| ctx.time() < limit)
    }

    fn until_score_above(self, limit: impl Into<Score>) -> impl Iterator<Item = E>
    where
        Self: Sized,
        E: Scored,
    {
        let lim = limit.into();
        self.skip_while(move |ctx| ctx.score().unwrap() < &lim)
    }

    fn until_score_below(self, limit: impl Into<Score>) -> impl Iterator<Item = E>
    where
        Self: Sized,
        E: Scored,
    {
        let lim = limit.into();
        self.skip_while(move |ctx| ctx.score().unwrap() > &lim)
    }

    fn until_score_equal(self, limit: impl Into<Score>) -> Option<E>
    where
        Self: Sized,
        E: Scored,
    {
        let limit = limit.into();
        self.skip_while(move |ctx| ctx.score().unwrap() != &limit)
            .take(1)
            .last()
    }

    fn until_converged(self, window: usize, epsilon: f32) -> impl Iterator<Item = E>
    where
        Self: Sized,
        E: Scored,
    {
        assert!(window > 0, "Window size must be greater than 0");
        assert!(epsilon >= 0.0, "Epsilon must be non-negative");

        ConverganceIterator {
            iter: self,
            history: VecDeque::new(),
            window,
            epsilon,
            done: false,
            _phantom: std::marker::PhantomData,
        }
    }
}

struct ConverganceIterator<I, C, E>
where
    I: Iterator<Item = E>,
    C: Chromosome,
    E: Epoch<Chromosome = C>,
{
    iter: I,
    history: VecDeque<f32>,
    window: usize,
    epsilon: f32,
    done: bool,
    _phantom: std::marker::PhantomData<C>,
}

impl<I, C, E> Iterator for ConverganceIterator<I, C, E>
where
    I: Iterator<Item = E>,
    C: Chromosome,
    E: Scored + Epoch<Chromosome = C>,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next_ctx = self.iter.next()?;
        let score = next_ctx.score().unwrap().as_f32();

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
