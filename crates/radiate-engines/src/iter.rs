use crate::{Generation, GeneticEngine};
use radiate_core::{
    Chromosome, Engine, Epoch, Objective, Optimize, Score,
    engine::{Context, EngineIter},
    objectives::Scored,
};
use std::{collections::VecDeque, time::Duration};

// impl<E: Engine> EngineExt for E {}

// /// Enhanced iterator for engines
// pub struct EngineIterator<E: Engine> {
//     engine: E,
// }

// pub struct EngineIterator<E, P>
// where
//     E: Engine<P>,
//     P: Epoch,
// {
//     pub(crate) engine: E,
//     _phantom: std::marker::PhantomData<P>,
// }

// impl<E, P> EngineIterator<E, P>
// where
//     P: Epoch,
//     E: Engine<P>,
// {
//     pub fn new(engine: E) -> Self {
//         EngineIterator {
//             engine,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

// impl<P, E> Iterator for EngineIterator<E, P>
// where
//     P: Epoch,
//     E: Engine<P>,
// {
//     type Item = P;

//     fn next(&mut self) -> Option<Self::Item> {
//         Some(self.engine.evolve())
//     }
// }

impl<I, C, T, E> EngineIteratorExt<C, T, E> for I
where
    I: Iterator<Item = E>,
    C: Chromosome,
    T: Clone,
    E: Epoch + for<'a> From<&'a Context<C, T>>,
{
}

pub trait EngineIteratorExt<C, T, E>: Iterator<Item = E>
where
    C: Chromosome,
    T: Clone,
    E: Epoch,
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

    fn until_score(self, limit: impl Into<Score>) -> impl Iterator<Item = E>
    where
        Self: Sized,
        E: Scored,
    {
        let lim = limit.into();
        self.skip_while(move |ctx| match ctx.objective() {
            Objective::Single(obj) => match obj {
                Optimize::Minimize => ctx.score().unwrap() >= &lim,
                Optimize::Maximize => ctx.score().unwrap() <= &lim,
            },
            Objective::Multi(objs) => {
                let mut all_pass = true;
                for (i, score) in ctx.score().unwrap().iter().enumerate() {
                    let passed = match objs[i] {
                        Optimize::Minimize => score >= &lim[i],
                        Optimize::Maximize => score <= &lim[i],
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
        }
    }
}

struct ConverganceIterator<I, E>
where
    I: Iterator<Item = E>,
    E: Epoch,
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
    E: Scored + Epoch,
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
