use super::{Chromosome, Codex, EngineError, Genotype, Score};
use std::sync::Arc;

pub trait Problem<C: Chromosome, T>: Send + Sync {
    fn encode(&self) -> Result<Genotype<C>, EngineError>;
    fn decode(&self, genotype: &Genotype<C>) -> Result<T, EngineError>;
    fn eval(&self, individual: &Genotype<C>) -> Result<Score, EngineError>;
}

pub(crate) struct EngineProblem<C, T>
where
    C: Chromosome,
{
    pub codex: Option<Arc<dyn Codex<C, T>>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
}

impl<C: Chromosome, T> Problem<C, T> for EngineProblem<C, T> {
    fn encode(&self) -> Result<Genotype<C>, EngineError> {
        match &self.codex {
            Some(codex) => Ok(codex.encode()),
            None => Err(EngineError::ProblemError(
                "Codex is not set (Encoding)".to_string(),
            )),
        }
    }

    fn decode(&self, genotype: &Genotype<C>) -> Result<T, EngineError> {
        match &self.codex {
            Some(codex) => Ok(codex.decode(genotype)),
            None => Err(EngineError::ProblemError(
                "Codex is not set (Decoding)".to_string(),
            )),
        }
    }

    fn eval(&self, individual: &Genotype<C>) -> Result<Score, EngineError> {
        let phenotype = self.decode(individual)?;
        match &self.fitness_fn {
            Some(fitness_fn) => Ok((fitness_fn)(phenotype)),
            None => Err(EngineError::ProblemError(
                "Fitness function is not set (FitnessFn)".to_string(),
            )),
        }
    }
}

unsafe impl<C: Chromosome, T> Send for EngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for EngineProblem<C, T> {}
