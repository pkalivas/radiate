pub mod audit;
pub mod evaluate;
pub mod filter;
pub mod front;
pub mod recombine;
pub mod speciate;

pub use audit::AuditStep;
pub use evaluate::EvaluateStep;
pub use filter::FilterStep;
pub use front::FrontStep;
pub use recombine::RecombineStep;
pub use speciate::SpeciateStep;

use crate::{Chromosome, EngineContext, GeneticEngineParams};

pub trait EngineStep<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized;
    fn execute(&self, context: &mut EngineContext<C, T>);
}
