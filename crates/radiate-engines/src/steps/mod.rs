pub mod audit;
pub mod evaluate;
pub mod filter;
pub mod front;
pub mod recombine;
pub mod speciate;

pub use audit::*;
pub use evaluate::*;
pub use filter::*;
pub use front::*;

use radiate_core::{Chromosome, Ecosystem, MetricSet};
use radiate_error::Result;

pub use recombine::*;
pub use speciate::*;

pub trait EngineStep<C>
where
    C: Chromosome,
{
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
            .split("<")
            .next()
            .unwrap_or(std::any::type_name::<Self>())
            .split("::")
            .last()
            .unwrap_or("Unknown Step")
    }

    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()>;
}
