use super::{Chromosome, Codex, Genotype, ProblemError, Score};
use std::sync::Arc;

pub trait Problem<C: Chromosome, T>: Send + Sync {
    fn encode(&self) -> Result<Genotype<C>, ProblemError>;
    fn decode(&self, genotype: &Genotype<C>) -> Result<T, ProblemError>;
    fn eval(&self, individual: &Genotype<C>) -> Result<Score, ProblemError>;
}

pub(crate) struct EngineProblem<C, T>
where
    C: Chromosome,
{
    pub codex: Option<Arc<dyn Codex<C, T>>>,
    pub fitness_fn: Option<Arc<dyn Fn(T) -> Score + Send + Sync>>,
}

impl<C: Chromosome, T> Problem<C, T> for EngineProblem<C, T> {
    fn encode(&self) -> Result<Genotype<C>, ProblemError> {
        match &self.codex {
            Some(codex) => Ok(codex.encode()),
            None => Err(ProblemError::EncodingError("Codex is not set".to_string())),
        }
    }

    fn decode(&self, genotype: &Genotype<C>) -> Result<T, ProblemError> {
        match &self.codex {
            Some(codex) => Ok(codex.decode(genotype)),
            None => Err(ProblemError::DecodingError("Codex is not set".to_string())),
        }
    }

    fn eval(&self, individual: &Genotype<C>) -> Result<Score, ProblemError> {
        let phenotype = self.decode(individual)?;
        match &self.fitness_fn {
            Some(fitness_fn) => Ok((fitness_fn)(phenotype)),
            None => Err(ProblemError::EvaluationError(
                "Fitness function is not set".to_string(),
            )),
        }
    }
}

unsafe impl<C: Chromosome, T> Send for EngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for EngineProblem<C, T> {}
