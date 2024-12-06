use crate::engines::genome::population::Population;
use crate::timer::Timer;
use crate::{random_provider, subset, Chromosome, Gene, Genotype, Metric, Phenotype};

pub enum AlterType {
    Mutator,
    Crossover,
    Alterer,
}

pub trait Alter<C: Chromosome> {
    fn name(&self) -> &'static str;
    fn rate(&self) -> f32;
    fn alter_type(&self) -> AlterType;

    fn alter(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric> {
        let mut metrics = Vec::new();
        let timer = Timer::new();
        let mut count = 0;

        match self.alter_type() {
            AlterType::Mutator => {
                for phenotype in population.iter_mut() {
                    let genotype = phenotype.genotype_mut();

                    let mutation_count = self.mutate_genotype(genotype);

                    if mutation_count > 0 {
                        phenotype.generation = generation;
                        phenotype.score = None;
                        count += mutation_count;
                    }
                }

                let mut new_metric = Metric::new_operations(self.name());
                new_metric.add_value(count as f32);
                new_metric.add_duration(timer.duration());

                metrics.push(new_metric);
            }
            AlterType::Crossover => {
                for i in 0..population.len() {
                    if random_provider::random::<f32>() < self.rate() {
                        let parent_indexes = subset::individual_indexes(i, population.len(), 2);
                        count += self.cross(population, &parent_indexes, generation);
                    }
                }

                let mut new_metric = Metric::new_operations(self.name());
                new_metric.add_value(count as f32);
                new_metric.add_duration(timer.duration());

                metrics.push(new_metric);
            }
            AlterType::Alterer => {}
        }

        metrics
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
