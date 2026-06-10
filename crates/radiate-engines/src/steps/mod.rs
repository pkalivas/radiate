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

use radiate_utils::ToSnakeCase;
pub use recombine::*;
pub use speciate::*;

pub trait EngineStep<C>: Send + Sync
where
    C: Chromosome,
{
    fn name(&self) -> &'static str {
        let name = std::any::type_name::<Self>()
            .split("<")
            .next()
            .unwrap_or(std::any::type_name::<Self>())
            .split("::")
            .last()
            .unwrap_or("Unknown Step");

        let snake_case = name.to_snake_case();

        let mut parts = snake_case
            .split('_')
            .filter(|part| !part.is_empty() && !part.contains("step"))
            .collect::<Vec<_>>();

        parts.insert(0, "step");
        let result = parts.join(".");

        radiate_utils::intern!(result)
    }

    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()>;
}
