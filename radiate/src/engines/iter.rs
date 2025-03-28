use super::{Chromosome, EngineContext, GeneticEngine, Phenotype};

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
        })
    }
}
