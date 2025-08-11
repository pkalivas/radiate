use super::{Chromosome, Codec, Genotype, Score};
use std::sync::Arc;

/// [Problem] represents the interface for the fitness function or evaluation and encoding/decoding
/// of a genotype to a phenotype within the genetic algorithm framework.
///
/// To run the genetic algorithm the three things that must be supplied are the encoding & decoding of
/// the [Genotype] and the fitness function. [Problem] wraps all three into a
/// single trait that can be supplied to the engine builder.
pub trait Problem<C: Chromosome, T>: Send + Sync {
    fn encode(&self) -> Genotype<C>;
    fn decode(&self, genotype: &Genotype<C>) -> T;
    fn eval(&self, individual: &Genotype<C>) -> Score;
    fn eval_batch(&self, individuals: &[Genotype<C>]) -> Vec<Score> {
        individuals.iter().map(|ind| self.eval(ind)).collect()
    }
}

/// [EngineProblem] is a generic, base level concrete implementation of the [Problem] trait that is the
/// default problem used by the engine if none other is specified during building. We take the
/// [Codec] and the fitness function from the user and simply wrap them into this struct.
pub struct EngineProblem<C, T>
where
    C: Chromosome,
{
    pub codec: Arc<dyn Codec<C, T>>,
    pub fitness_fn: Arc<dyn Fn(T) -> Score + Send + Sync>,
}

impl<C: Chromosome, T> Problem<C, T> for EngineProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        let phenotype = self.decode(individual);
        (self.fitness_fn)(phenotype)
    }
}

unsafe impl<C: Chromosome, T> Send for EngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for EngineProblem<C, T> {}

pub struct BatchEngineProblem<C, T>
where
    C: Chromosome,
{
    pub codec: Arc<dyn Codec<C, T>>,
    pub batch_fitness_fn: Arc<dyn Fn(&[T]) -> Vec<Score> + Send + Sync>,
}

impl<C: Chromosome, T> Problem<C, T> for BatchEngineProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        let phenotype = self.decode(individual);
        let scores = (self.batch_fitness_fn)(&[phenotype]);

        // Cloning a score is a lightweight operation - the internal of a score is a Arc<[f32]>
        // This function will likely never be called anyways as we expect `eval_batch` to be used.
        scores[0].clone()
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> Vec<Score> {
        let phenotypes = individuals
            .iter()
            .map(|genotype| self.decode(genotype))
            .collect::<Vec<T>>();
        (self.batch_fitness_fn)(&phenotypes)
    }
}

unsafe impl<C: Chromosome, T> Send for BatchEngineProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for BatchEngineProblem<C, T> {}
