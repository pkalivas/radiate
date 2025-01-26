use std::sync::Arc;

use super::{Chromosome, Codex, Genotype, Score};

pub trait Problem<C: Chromosome, T>: Send + Sync {
    fn encode(&self) -> Genotype<C>;
    fn decode(&self, genotype: &Genotype<C>) -> T;
    fn eval(&self, individual: &Genotype<C>) -> Score;
}

impl<C: Chromosome, T> Codex<C, T> for dyn Problem<C, T> {
    fn encode(&self) -> Genotype<C> {
        Problem::encode(self)
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        Problem::decode(self, genotype)
    }
}

pub struct DefaultProblem<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub codex: Arc<Box<dyn Codex<C, T>>>,
    pub fitness_fn: Arc<dyn Fn(T) -> Score + Send + Sync>,
}

unsafe impl<C, T> Send for DefaultProblem<C, T>
where
    C: Chromosome,
    T: Clone,
{
}

unsafe impl<C, T> Sync for DefaultProblem<C, T>
where
    C: Chromosome,
    T: Clone,
{
}

impl<C, T> Problem<C, T> for DefaultProblem<C, T>
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
