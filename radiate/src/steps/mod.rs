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

use crate::{Chromosome, GeneticEngineParams, Metric, Population, Species};

pub trait EngineStep<C, T>
where
    C: Chromosome,
    T: Clone + Send,
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

    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized;

    fn execute(
        &self,
        generation: usize,
        population: &mut Population<C>,
        species: &mut Vec<Species<C>>,
    ) -> Vec<Metric>;
}
