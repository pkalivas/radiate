use super::Alter;
use crate::{random_provider, timer::Timer, Chromosome, Gene, Genotype, Metric, Population};

pub trait Mutate<C: Chromosome>: Alter<C> {
    #[inline]
    fn mutate(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric> {
        let timer = Timer::new();
        let mut count = 0;

        for phenotype in population.iter_mut() {
            let genotype = phenotype.genotype_mut();

            let mutation_count = self.mutate_genotype(genotype);

            if mutation_count > 0 {
                phenotype.generation = generation;
                phenotype.score = None;
                count += mutation_count;
            }
        }

        vec![Metric::new_operations(
            self.name(),
            count as f32,
            timer.duration(),
        )]
    }

    #[inline]
    fn mutate_genotype(&self, genotype: &mut Genotype<C>) -> i32 {
        let mut count = 0;
        for chromosome in genotype.iter_mut() {
            count += self.mutate_chromosome(chromosome);
        }

        count
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::random::<f32>() < self.rate() {
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
