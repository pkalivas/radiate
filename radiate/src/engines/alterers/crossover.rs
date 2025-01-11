use crate::{
    indexes, random_provider, timer::Timer, Chromosome, Gene, Metric, Phenotype, Population,
};

use super::Alter;

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
pub trait Crossover<C: Chromosome>: Alter<C> {
    #[inline]
    fn crossover(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric> {
        let timer = Timer::new();
        let mut count = 0;

        for i in 0..population.len() {
            if random_provider::random::<f32>() < self.rate() {
                let parent_indexes = indexes::individual_indexes(i, population.len(), 2);
                count += self.cross(population, &parent_indexes, generation);
            }
        }

        let mut new_metric = Metric::new_operations(self.name());
        new_metric.add_value(count as f32);
        new_metric.add_duration(timer.duration());

        vec![new_metric]
    }

    #[inline]
    fn cross(
        &self,
        population: &mut Population<C>,
        parent_indexes: &[usize],
        generation: i32,
    ) -> i32 {
        let index_one = parent_indexes[0];
        let index_two = parent_indexes[1];

        let mut geno_one = population[index_one].genotype().clone();
        let mut geno_two = population[index_two].genotype().clone();

        let chromosome_index =
            random_provider::random::<usize>() % std::cmp::min(geno_one.len(), geno_two.len());

        let chrom_one = &mut geno_one[chromosome_index];
        let chrom_two = &mut geno_two[chromosome_index];

        let cross_count = self.cross_chromosomes(chrom_one, chrom_two);

        if cross_count > 0 {
            population[index_one] = Phenotype::from_genotype(geno_one, generation);
            population[index_two] = Phenotype::from_genotype(geno_two, generation);
        }

        cross_count
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C) -> i32 {
        let rate = self.rate();
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

        cross_count
    }
}
