use super::{Chromosome, EngineCell, EngineContext, GeneticEngine};

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
    cell: EngineCell<C, T>,
    limits: Option<Vec<Box<dyn Fn(&EngineContext<C, T>) -> bool>>>,
}

impl<'a, C, T> EngineIterator<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    pub fn new(engine: &'a GeneticEngine<C, T>) -> Self {
        let cell = EngineCell::new(engine.start());
        EngineIterator {
            engine,
            cell,
            limits: None,
        }
    }

    pub fn limit(&mut self, limit: impl Fn(&EngineContext<C, T>) -> bool + 'static) -> &mut Self {
        if self.limits.is_none() {
            self.limits = Some(vec![Box::new(limit)]);
        } else if let Some(ref mut limits) = self.limits {
            limits.push(Box::new(limit));
        }

        self
    }
}

impl<'a, C, T> Iterator for EngineIterator<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    type Item = EngineCell<C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut ctx = self.cell.write();

        if let Some(limits) = &self.limits {
            for limit in limits {
                if !limit(&ctx) {
                    return None;
                }
            }
        }

        self.engine.next(&mut ctx);

        Some(self.cell.clone())
    }
}
