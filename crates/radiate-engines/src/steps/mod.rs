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

        if let Some(interned) = radiate_utils::try_get_interned_str(name) {
            return interned;
        }

        let snake_case_name = radiate_utils::intern_name_as_snake_case(name);

        let mut parts = snake_case_name
            .split('_')
            .filter(|part| !part.is_empty() && !part.contains("step"))
            .collect::<Vec<_>>();

        parts.insert(0, "step");

        radiate_utils::intern_kv_pair(name, radiate_utils::intern!(parts.join(".")))
    }

    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()>;
}
