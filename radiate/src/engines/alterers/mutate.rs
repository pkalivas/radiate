use super::{AlterResult, IntoAlter};
use crate::{Chromosome, Gene, Genotype, Population, random_provider};

pub trait Mutate<C: Chromosome>: IntoAlter<C> {
    #[inline]
    fn mutate(&self, population: &mut Population<C>, generation: i32, rate: f32) -> AlterResult {
        let mut count = 0;

        for phenotype in population.iter_mut() {
            let genotype = phenotype.genotype_mut();

            let mutation_count = self.mutate_genotype(genotype, rate);

            if mutation_count > 0 {
                phenotype.generation = generation;
                phenotype.score = None;
                count += mutation_count;
            }
        }

        AlterResult(count, Vec::new())
    }

    #[inline]
    fn mutate_genotype(&self, genotype: &mut Genotype<C>, rate: f32) -> i32 {
        let mut count = 0;
        for chromosome in genotype.iter_mut() {
            count += self.mutate_chromosome(chromosome, rate);
        }

        count
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> i32 {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::random::<f32>() < rate {
                *gene = self.mutate_gene(gene);
                count += 1;
            }
        }

        count
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        gene.new_instance()
    }
}
