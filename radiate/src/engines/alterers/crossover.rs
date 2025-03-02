use super::{AlterResult, IntoAlter};
use crate::{Chromosome, Gene, Phenotype, Population, indexes, random_provider};

/// The `Crossover` trait is used to define the crossover operation for a genetic algorithm.
///
/// In a genetic algorithm, crossover is a genetic operator used to vary the
/// programming of a chromosome or chromosomes from one generation to the next.
/// It is analogous to reproduction and biological crossover, upon which genetic algorithms are based.
///
/// A `Crossover` typically takes two parent chromosomes and produces two or more offspring chromosomes.
/// This trait allows you to define your own crossover operation on either the entire population
/// or a subset of the population. If a struct implements the `Crossover` trait but does not override
/// any of the methods, the default implementation will perform a simple crossover operation on the
/// entire population. This is the case with the `UniformCrossover` struct.
pub trait Crossover<C: Chromosome>: IntoAlter<C> {
    #[inline]
    fn crossover(
        &self,
        population: &mut Population<C>,
        generation: usize,
        rate: f32,
    ) -> AlterResult {
        let mut result = AlterResult::default();

        for i in 0..population.len() {
            if random_provider::random::<f32>() < rate {
                let parent_indexes = indexes::individual_indexes(i, population.len(), 2);
                let cross_result = self.cross(population, &parent_indexes, generation, rate);
                result.merge(cross_result);
            }
        }

        result
    }

    #[inline]
    fn cross(
        &self,
        population: &mut Population<C>,
        parent_indexes: &[usize],
        generation: usize,
        rate: f32,
    ) -> AlterResult {
        let mut result = AlterResult::default();

        let index_one = parent_indexes[0];
        let index_two = parent_indexes[1];

        let mut geno_one = population[index_one].genotype().clone();
        let mut geno_two = population[index_two].genotype().clone();

        let max_idx = std::cmp::min(geno_one.len(), geno_two.len());
        let chromosome_index = random_provider::random_range(0..max_idx);

        let chrom_one = &mut geno_one[chromosome_index];
        let chrom_two = &mut geno_two[chromosome_index];

        let cross_result = self.cross_chromosomes(chrom_one, chrom_two, rate);

        if cross_result.count() > 0 {
            population[index_one] = Phenotype::from_genotype(geno_one, generation);
            population[index_two] = Phenotype::from_genotype(geno_two, generation);
        }

        result.merge(cross_result);
        result
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                let new_gene_one = gene_one.with_allele(gene_two.allele());
                let new_gene_two = gene_two.with_allele(gene_one.allele());

                chrom_one.set_gene(i, new_gene_one);
                chrom_two.set_gene(i, new_gene_two);

                cross_count += 1;
            }
        }

        cross_count.into()
    }
}
