pub mod gaussian_mutator;
pub mod numeric_mutator;
pub mod swap_mutator;
pub mod uniform_mutator;

pub use numeric_mutator::NumericMutator;
pub use swap_mutator::SwapMutator;
pub use uniform_mutator::UniformMutator;

use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::genotype::Genotype;
use crate::{random_provider, Chromosome};

pub trait Mutate<C: Chromosome> {
    fn mutate_rate(&self) -> f32;

    fn name(&self) -> &'static str;

    #[inline]
    fn mutate_genotype(&self, genotype: &mut Genotype<C>, range: i32) -> i32 {
        let mut count = 0;
        for chromosome in genotype.iter_mut() {
            if random_provider::random::<i32>() < range {
                count += self.mutate_chromosome(chromosome, range);
            }
        }

        count
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, range: i32) -> i32 {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::random::<i32>() < range {
                *gene = self.mutate_gene(gene);
                count += 1;
            }
        }

        count
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::GeneType) -> C::GeneType {
        gene.new_instance()
    }
}
