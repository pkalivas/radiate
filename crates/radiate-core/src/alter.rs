use crate::{Chromosome, Gene, Genotype, Metric, Population, indexes, random_provider};

/// This is the main trait that is used to define the different types of alterations that can be
/// performed on a population. The `Alter` trait is used to define the `alter` method that is used
/// to perform the alteration on the population. The `alter` method takes a mutable reference to
/// the population and a generation number as parameters. The `alter` method returns a vector of
/// `Metric` objects that represent the metrics that were collected during the alteration process.
///
/// An 'Alter' in a traditional genetic algorithm is a process that modifies the population of
/// individuals in some way. This can include operations such as mutation or crossover. The goal of
/// an alter is to introduce new genetic material into the population, which can help to improve
/// the overall fitness of the population. In a genetic algorithm, the alter is typically
/// performed on a subset of the population, rather than the entire population. This allows for
/// more targeted modifications to be made, which can help to improve the overall performance of
/// the algorithm. The alter is an important part of the genetic algorithm process, as it helps
/// to ensure that the population remains diverse and that new genetic material is introduced
/// into the population. This can help to improve the overall performance of the algorithm and
/// ensure that the population remains healthy and diverse.
///
/// In `radiate` the `alter` trait performs similar operations to a traditional genetic algorithm,
/// but it is designed to be more flexible and extensible. Because an `Alter` can be of type `Mutate`
/// or `Crossover`, it is abstracted out of those core traits into this trait.
pub trait Alter<C: Chromosome>: Send + Sync {
    fn alter(&self, population: &mut Population<C>, generation: usize) -> Vec<Metric>;
}

/// The `AlterResult` struct is used to represent the result of an
/// alteration operation. It contains the number of operations
/// performed and a vector of metrics that were collected
/// during the alteration process.
#[derive(Default)]
pub struct AlterResult(pub usize, pub Option<Vec<Metric>>);

impl AlterResult {
    pub fn count(&self) -> usize {
        self.0
    }

    pub fn metrics(&self) -> Option<&Vec<Metric>> {
        self.1.as_ref()
    }

    pub fn merge(&mut self, other: AlterResult) {
        let AlterResult(other_count, other_metrics) = other;

        self.0 += other_count;
        if let Some(metrics) = other_metrics {
            if let Some(self_metrics) = &mut self.1 {
                self_metrics.extend(metrics);
            } else {
                self.1 = Some(metrics);
            }
        }
    }
}

impl Into<AlterResult> for usize {
    fn into(self) -> AlterResult {
        AlterResult(self, None)
    }
}

impl Into<AlterResult> for (usize, Vec<Metric>) {
    fn into(self) -> AlterResult {
        AlterResult(self.0, Some(self.1))
    }
}

impl Into<AlterResult> for (usize, Metric) {
    fn into(self) -> AlterResult {
        AlterResult(self.0, Some(vec![self.1]))
    }
}

/// The `AlterAction` enum is used to represent the different
/// types of alterations that can be performed on a
/// population - It can be either a mutation or a crossover operation.
pub enum AlterAction<C: Chromosome> {
    Mutate(&'static str, f32, Box<dyn Mutate<C>>),
    Crossover(&'static str, f32, Box<dyn Crossover<C>>),
}

impl<C: Chromosome> Alter<C> for AlterAction<C> {
    fn alter(&self, population: &mut Population<C>, generation: usize) -> Vec<Metric> {
        match &self {
            AlterAction::Mutate(name, rate, m) => {
                let timer = std::time::Instant::now();
                let AlterResult(count, metrics) = m.mutate(population, generation, *rate);
                let metric = Metric::new_operations(name, count, timer.elapsed());

                match metrics {
                    Some(metrics) => metrics.into_iter().chain(std::iter::once(metric)).collect(),
                    None => vec![metric],
                }
            }
            AlterAction::Crossover(name, rate, c) => {
                let timer = std::time::Instant::now();
                let AlterResult(count, metrics) = c.crossover(population, generation, *rate);
                let metric = Metric::new_operations(name, count, timer.elapsed());

                match metrics {
                    Some(metrics) => metrics.into_iter().chain(std::iter::once(metric)).collect(),
                    None => vec![metric],
                }
            }
        }
    }
}

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
pub trait Crossover<C: Chromosome>: Send + Sync {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    fn rate(&self) -> f32 {
        1.0
    }

    fn alterer(self) -> AlterAction<C>
    where
        Self: Sized + 'static,
    {
        AlterAction::Crossover(self.name(), self.rate(), Box::new(self))
    }

    #[inline]
    fn crossover(
        &self,
        population: &mut Population<C>,
        generation: usize,
        rate: f32,
    ) -> AlterResult {
        let mut result = AlterResult::default();

        for i in 0..population.len() {
            if random_provider::random::<f32>() < rate && population.len() > 3 {
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

        let (one, two) = population.get_pair_mut(parent_indexes[0], parent_indexes[1]);

        let cross_result = {
            let geno_one = one.genotype_mut();
            let geno_two = two.genotype_mut();

            let min_len = std::cmp::min(geno_one.len(), geno_two.len());
            let chromosome_index = random_provider::range(0..min_len);

            let chrom_one = &mut geno_one[chromosome_index];
            let chrom_two = &mut geno_two[chromosome_index];

            self.cross_chromosomes(chrom_one, chrom_two, rate)
        };

        if cross_result.count() > 0 {
            one.invalidate(generation);
            two.invalidate(generation);
            result.merge(cross_result);
        }

        result
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < rate {
                let gene_one = chrom_one.get(i);
                let gene_two = chrom_two.get(i);

                let new_gene_one = gene_one.with_allele(gene_two.allele());
                let new_gene_two = gene_two.with_allele(gene_one.allele());

                chrom_one.set(i, new_gene_one);
                chrom_two.set(i, new_gene_two);

                cross_count += 1;
            }
        }

        cross_count.into()
    }
}

pub trait Mutate<C: Chromosome>: Send + Sync {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    fn rate(&self) -> f32 {
        1.0
    }

    fn alterer(self) -> AlterAction<C>
    where
        Self: Sized + 'static,
    {
        AlterAction::Mutate(self.name(), self.rate(), Box::new(self))
    }

    #[inline]
    fn mutate(&self, population: &mut Population<C>, generation: usize, rate: f32) -> AlterResult {
        let mut result = AlterResult::default();

        for phenotype in population.iter_mut() {
            let mutate_result = self.mutate_genotype(&mut phenotype.genotype_mut(), rate);

            if mutate_result.count() > 0 {
                phenotype.invalidate(generation);
            }

            result.merge(mutate_result);
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
