use super::{Chromosome, EngineContext, GeneticEngine, Phenotype};
use std::{collections::VecDeque, time::Duration};

pub trait EngineIter<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn iter(&self) -> EngineIterator<C, T>;
}

impl<C, T> EngineIter<C, T> for GeneticEngine<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn iter(&self) -> EngineIterator<C, T> {
        EngineIterator::new(self)
    }
}

pub struct EngineIterator<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    engine: &'a GeneticEngine<C, T>,
    ctx: EngineContext<C, T>,
}

impl<'a, C, T> EngineIterator<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    pub fn new(engine: &'a GeneticEngine<C, T>) -> Self {
        let ctx = engine.start();

        EngineIterator {
            engine,
            ctx: ctx.clone(),
        }
    }
}

impl<'a, C, T> Iterator for EngineIterator<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    type Item = EngineContext<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.engine.next(&mut self.ctx);

        Some(EngineContext {
            population: self
                .ctx
                .population
                .iter()
                .map(|phenotype| Phenotype::clone(phenotype))
                .collect(),
            best: self.ctx.best.clone(),
            index: self.ctx.index,
            timer: self.ctx.timer.clone(),
            metrics: self.ctx.metrics.clone(),
            score: self.ctx.score.clone(),
            front: self.ctx.front.clone(),
            species: self.ctx.species.clone(),
            objective: self.ctx.objective.clone(),
            decoder: self.ctx.decoder.clone(),
        })
    }
}

pub trait EngineIteratorExt<C, T>: Iterator<Item = EngineContext<C, T>>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn map_best(self) -> impl Iterator<Item = T>
    where
        Self: Sized,
    {
        self.map(|ctx| ctx.best().clone())
    }

    fn until_score_below(self, threshold: f32) -> impl Iterator<Item = EngineContext<C, T>>
    where
        Self: Sized,
    {
        self.take_while(move |ctx| ctx.score().as_f32() > threshold)
    }

    fn until_score_above(self, threshold: f32) -> impl Iterator<Item = EngineContext<C, T>>
    where
        Self: Sized,
    {
        self.take_while(move |ctx| ctx.score().as_f32() < threshold)
    }

    fn until_seconds(self, limit: f64) -> impl Iterator<Item = EngineContext<C, T>>
    where
        Self: Sized,
    {
        self.take_while(move |ctx| ctx.seconds() < limit)
    }

    fn until_duration(self, limit: Duration) -> impl Iterator<Item = EngineContext<C, T>>
    where
        Self: Sized,
    {
        self.take_while(move |ctx| ctx.duration() < limit)
    }

    fn until_converged(
        self,
        window: usize,
        epsilon: f32,
    ) -> impl Iterator<Item = EngineContext<C, T>>
    where
        Self: Sized,
    {
        struct ConverganceIterator<I, C, T>
        where
            C: Chromosome + 'static,
            T: Clone + Send + 'static,
            I: Iterator<Item = EngineContext<C, T>>,
        {
            iter: I,
            history: VecDeque<f32>,
            window: usize,
            epsilon: f32,
            done: bool,
        }

        impl<I, C, T> Iterator for ConverganceIterator<I, C, T>
        where
            C: Chromosome + 'static,
            T: Clone + Send + 'static,
            I: Iterator<Item = EngineContext<C, T>>,
        {
            type Item = EngineContext<C, T>;

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

        ConverganceIterator {
            iter: self,
            history: VecDeque::new(),
            window,
            epsilon,
            done: false,
        }
    }

    fn only_improvements(self) -> impl Iterator<Item = EngineContext<C, T>>
    where
        Self: Sized,
    {
        struct ImprovementFilter<I, C, T>
        where
            I: Iterator<Item = EngineContext<C, T>>,
            C: Chromosome + 'static,
            T: Clone + Send + 'static,
        {
            iter: I,
            best: Option<f32>,
        }

        impl<I, C, T> Iterator for ImprovementFilter<I, C, T>
        where
            I: Iterator<Item = EngineContext<C, T>>,
            C: Chromosome + 'static,
            T: Clone + Send + 'static,
        {
            type Item = EngineContext<C, T>;

            fn next(&mut self) -> Option<Self::Item> {
                while let Some(ctx) = self.iter.next() {
                    let score = ctx.score().as_f32();
                    let improved = match self.best {
                        None => true,
                        Some(best_so_far) => ctx.objective().is_better(&score, &best_so_far),
                    };

                    if improved {
                        self.best = Some(score);
                        return Some(ctx);
                    }
                }
                None
            }
        }

        ImprovementFilter {
            iter: self,
            best: None,
        }
    }

    fn until_stagnant_generations(
        self,
        max_stagnant: usize,
    ) -> impl Iterator<Item = EngineContext<C, T>>
    where
        Self: Sized,
    {
        struct Stagnation<I, C, T>
        where
            I: Iterator<Item = EngineContext<C, T>>,
            C: Chromosome + 'static,
            T: Clone + Send + 'static,
        {
            iter: I,
            best: Option<f32>,
            stagnant: usize,
            max_stagnant: usize,
        }

        impl<I, C, T> Iterator for Stagnation<I, C, T>
        where
            I: Iterator<Item = EngineContext<C, T>>,
            C: Chromosome + 'static,
            T: Clone + Send + 'static,
        {
            type Item = EngineContext<C, T>;

            fn next(&mut self) -> Option<Self::Item> {
                while let Some(ctx) = self.iter.next() {
                    let score = ctx.score().as_f32();
                    let improved = match self.best {
                        None => true,
                        Some(best) => ctx.objective().is_better(&score, &best),
                    };

                    if improved {
                        self.best = Some(score);
                        self.stagnant = 0;
                    } else {
                        self.stagnant += 1;
                    }

                    if self.stagnant >= self.max_stagnant {
                        return None;
                    }

                    return Some(ctx);
                }
                None
            }
        }

        Stagnation {
            iter: self,
            best: None,
            stagnant: 0,
            max_stagnant,
        }
    }
}

impl<I, C, T> EngineIteratorExt<C, T> for I
where
    I: Iterator<Item = EngineContext<C, T>>,
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
}
