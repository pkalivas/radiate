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

// TODO: Maybe make a codex for decoding to &Genoty<C>

// #[allow(dead_code)]
// pub(crate) struct GenotypeProblem<C>
// where
//     C: Chromosome,
// {
//     pub encoder: Arc<dyn Fn() -> Genotype<C>>,
//     pub fitness_fn: Arc<dyn Fn(&Genotype<C>) -> Score + Send + Sync>,
// }

// impl<C: Chromosome> Problem<C, Genotype<C>> for GenotypeProblem<C> {
//     fn encode(&self) -> Genotype<C> {
//         (self.encoder)()
//     }

//     fn decode(&self, genotype: &Genotype<C>) -> Genotype<C> {
//         genotype.clone()
//     }

//     fn eval(&self, individual: &Genotype<C>) -> Score {
//         (self.fitness_fn)(individual)
//     }
// }

// unsafe impl<C: Chromosome> Send for GenotypeProblem<C> {}
// unsafe impl<C: Chromosome> Sync for GenotypeProblem<C> {}
