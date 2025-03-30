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

pub struct StepWrapper<C: Chromosome, T: Clone + Send>(
    pub EngineStepType,
    pub Box<dyn EngineStep<C, T>>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStepType {
    Evaluate,
    Speciate,
    Recombine,
    Filter,
    Front,
    Audit,
}

impl EngineStepType {
    pub fn name(self) -> &'static str {
        match self {
            EngineStepType::Evaluate => "evaluate",
            EngineStepType::Speciate => "speciate",
            EngineStepType::Recombine => "recombine",
            EngineStepType::Filter => "filter",
            EngineStepType::Front => "front",
            EngineStepType::Audit => "audit",
        }
    }
}
