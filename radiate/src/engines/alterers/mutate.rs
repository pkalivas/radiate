use super::{AlterResult, IntoAlter};
use crate::{Chromosome, Gene, Genotype, Population, random_provider};

pub trait Mutate<C: Chromosome>: IntoAlter<C> {
    #[inline]
    fn mutate(&self, population: &mut Population<C>, generation: usize, rate: f32) -> AlterResult {
        let mut result = AlterResult::default();

        for phenotype in population.iter_mut() {
            let genotype = phenotype.genotype_mut();

            let mutate_result = self.mutate_genotype(genotype, rate);

            if mutate_result.count() > 0 {
                phenotype.generation = generation;
                phenotype.score = None;

                result.merge(mutate_result);
            }
        }

        result
    }

    #[inline]
    fn mutate_genotype(&self, genotype: &mut Genotype<C>, rate: f32) -> AlterResult {
        let mut result = AlterResult::default();

        for chromosome in genotype.iter_mut() {
            let mutate_result = self.mutate_chromosome(chromosome, rate);
            result.merge(mutate_result);
        }

        result
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::random::<f32>() < rate {
                *gene = self.mutate_gene(gene);
                count += 1;
            }
        }

        count.into()
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        gene.new_instance()
    }
}
