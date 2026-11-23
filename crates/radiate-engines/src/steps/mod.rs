pub mod evaluate;
pub mod filter;
pub mod front;
pub mod metric;
pub mod recombine;
pub mod speciate;

pub use evaluate::*;
pub use filter::*;
pub use front::*;
pub use metric::*;

use radiate_core::{Chromosome, Ecosystem, MetricSet, SmallStr};
use radiate_error::Result;

pub use recombine::*;
pub use speciate::*;

use crate::EvolutionContext;

type EvolutionAction<C, T> = Box<dyn FnMut(&mut EvolutionContext<C, T>) -> Result<()> + Send>;

pub trait EngineStep<C>: Send + Sync
where
    C: Chromosome,
{
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()>;
}

pub(crate) struct EvolutionStep<C: Chromosome, T> {
    pub(crate) name: SmallStr,
    pub(crate) action: EvolutionAction<C, T>,
}

impl<C: Chromosome, T> EvolutionStep<C, T> {
    pub fn new(name: impl Into<SmallStr>, action: EvolutionAction<C, T>) -> Self {
        Self {
            name: name.into(),
            action,
        }
    }

    pub fn call(&mut self, context: &mut EvolutionContext<C, T>) -> Result<()> {
        (self.action)(context)
    }
}
