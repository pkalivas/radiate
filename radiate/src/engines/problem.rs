use super::{Chromosome, Codex, Genotype, Score};
use std::sync::Arc;

pub trait Problem<C: Chromosome, T>: Send + Sync {
    fn encode(&self) -> Genotype<C>;
    fn decode(&self, genotype: &Genotype<C>) -> T;
    fn eval(&self, individual: &Genotype<C>) -> Score;
}

pub(crate) struct EngineProblem<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub codex: Arc<dyn Codex<C, T>>,
    pub fitness_fn: Arc<dyn Fn(T) -> Score + Send + Sync>,
}

unsafe impl<C: Chromosome, T: Clone> Send for EngineProblem<C, T> {}
unsafe impl<C: Chromosome, T: Clone> Sync for EngineProblem<C, T> {}

impl<C, T> Problem<C, T> for EngineProblem<C, T>
where
    C: Chromosome,
    T: Clone,
{
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
