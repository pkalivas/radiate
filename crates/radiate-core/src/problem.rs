use super::{Chromosome, Codex, Genotype, Score};
use std::sync::Arc;

pub trait Problem<C: Chromosome, T>: Send + Sync {
    fn encode(&self) -> Genotype<C>;
    fn decode(&self, genotype: &Genotype<C>) -> T;
    fn eval(&self, individual: &Genotype<C>) -> Score;
}

pub struct EngineProblem<C, T>
where
    C: Chromosome,
{
    pub codex: Arc<dyn Codex<C, T>>,
    pub fitness_fn: Arc<dyn Fn(T) -> Score + Send + Sync>,
}

impl<C: Chromosome, T> Problem<C, T> for EngineProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codex.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codex.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        let phenotype = self.decode(individual);
        (self.fitness_fn)(phenotype)
    }
}

unsafe impl<C: Chromosome, T> Send for EngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for EngineProblem<C, T> {}
