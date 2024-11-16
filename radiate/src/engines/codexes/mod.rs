use super::genome::genes::gene::Gene;
use super::genome::genotype::Genotype;
use super::genome::phenotype::Phenotype;
use super::genome::population::Population;

pub mod bit_codex;
pub mod char_codex;
pub mod float_codex;
pub mod generic_codex;
pub mod int_codex;
pub mod subset_codex;

pub use bit_codex::*;
pub use char_codex::*;
pub use float_codex::*;
pub use generic_codex::*;
pub use int_codex::*;
pub use subset_codex::*;

pub trait Codex<G, A, T>
where
    G: Gene<G, A>,
{
    fn encode(&self) -> Genotype<G, A>;

    fn decode(&self, genotype: &Genotype<G, A>) -> T;

    fn spawn(&self, num: i32) -> Vec<T> {
        (0..num)
            .into_iter()
            .map(|_| self.decode(&self.encode()))
            .collect::<Vec<T>>()
    }

    fn spawn_genotypes(&self, num: i32) -> Vec<Genotype<G, A>> {
        (0..num)
            .into_iter()
            .map(|_| self.encode())
            .collect::<Vec<Genotype<G, A>>>()
    }

    fn spawn_population(&self, num: i32) -> Population<G, A> {
        (0..num)
            .into_iter()
            .map(|_| Phenotype::from_genotype(self.encode(), 0))
            .collect::<Population<G, A>>()
    }
}
