use crate::{Chromosome, Gene, Genotype, Metric, Population, math::indexes, random_provider};
use crate::{Lineage, LineageUpdate, Rate, metric};
use radiate_utils::{ToSnakeCase, intern};
use std::cell::{LazyCell, RefCell};
use std::iter::once;
use std::sync::Arc;

#[macro_export]
macro_rules! alters {
    ($($struct_instance:expr),* $(,)?) => {
        {
            let mut vec: Vec<Alterer<_>> = Vec::new();
            $(
                vec.push($struct_instance.alterer());
            )*
            vec
        }
    };
}

/// The [AlterResult] struct is used to represent the result of an
/// alteration operation. It contains the number of operations
/// performed and a vector of metrics that were collected
/// during the alteration process.
pub struct AlterResult(
    pub usize,
    pub Option<Vec<Metric>>,
    pub Option<Vec<LineageUpdate>>,
);

impl AlterResult {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn count(&self) -> usize {
        self.0
    }

    pub fn update_lineage(&mut self, lineage: impl Into<LineageUpdate>) {
        let lineage = lineage.into();
        if let Some(self_lineage) = &mut self.2 {
            self_lineage.push(lineage);
        } else {
            self.2 = Some(vec![lineage]);
        }
    }

    pub fn merge(&mut self, other: AlterResult) {
        let AlterResult(other_count, other_metrics, other_lineage) = other;

        self.0 += other_count;
        if let Some(metrics) = other_metrics {
            if let Some(self_metrics) = &mut self.1 {
                self_metrics.extend(metrics);
            } else {
                self.1 = Some(metrics);
            }
        }

        if let Some(lineage) = other_lineage {
            if let Some(self_lineage) = &mut self.2 {
                self_lineage.extend(lineage);
            } else {
                self.2 = Some(lineage);
            }
        }
    }
}

impl Default for AlterResult {
    fn default() -> Self {
        AlterResult(0, None, None)
    }
}

impl From<usize> for AlterResult {
    fn from(value: usize) -> Self {
        AlterResult(value, None, None)
    }
}

impl From<(usize, Vec<Metric>)> for AlterResult {
    fn from((count, metrics): (usize, Vec<Metric>)) -> Self {
        AlterResult(count, Some(metrics), None)
    }
}

impl From<(usize, Metric)> for AlterResult {
    fn from((count, metric): (usize, Metric)) -> Self {
        AlterResult(count, Some(vec![metric]), None)
    }
}

impl From<Metric> for AlterResult {
    fn from(value: Metric) -> Self {
        AlterResult(1, Some(vec![value]), None)
    }
}

impl From<(usize, LineageUpdate)> for AlterResult {
    fn from((count, lineage): (usize, LineageUpdate)) -> Self {
        AlterResult(count, None, Some(vec![lineage]))
    }
}

impl From<(usize, Vec<Metric>, Vec<LineageUpdate>)> for AlterResult {
    fn from((count, metrics, lineage): (usize, Vec<Metric>, Vec<LineageUpdate>)) -> Self {
        AlterResult(count, Some(metrics), Some(lineage))
    }
}

impl From<(Vec<Metric>, Vec<LineageUpdate>)> for AlterResult {
    fn from((metrics, lineage): (Vec<Metric>, Vec<LineageUpdate>)) -> Self {
        AlterResult(metrics.len(), Some(metrics), Some(lineage))
    }
}

/// The [Alterer] enum is used to represent the different
/// types of alterations that can be performed on a
/// population - It can be either a mutation or a crossover operation.

#[derive(Clone)]
pub enum Alterer<C: Chromosome> {
    Mutate(&'static str, Rate, Arc<dyn Mutate<C>>),
    Crossover(&'static str, Rate, Arc<dyn Crossover<C>>),
}

impl<C: Chromosome> Alterer<C> {
    pub fn name(&self) -> &str {
        match &self {
            Alterer::Mutate(name, _, _) => name,
            Alterer::Crossover(name, _, _) => name,
        }
    }

    pub fn rate(&self) -> &Rate {
        match &self {
            Alterer::Mutate(_, rate, _) => rate,
            Alterer::Crossover(_, rate, _) => rate,
        }
    }

    #[inline]
    pub fn alter(
        &self,
        population: &mut Population<C>,
        lineage: &mut Lineage,
        generation: usize,
    ) -> Vec<Metric> {
        match &self {
            Alterer::Mutate(name, rate, m) => {
                let (rate_value, rate_metric) = Self::rate_metric(generation, rate, name);

                let timer = std::time::Instant::now();
                let AlterResult(count, metrics, lineage_events) =
                    m.mutate(population, generation, rate_value);

                if let Some(lineage_events) = lineage_events {
                    lineage.extend(name, lineage_events);
                }

                let metric = metric!(name, (count, timer.elapsed()));

                match metrics {
                    Some(metrics) => metrics
                        .into_iter()
                        .chain(once(metric))
                        .chain(once(rate_metric))
                        .collect(),
                    None => vec![metric, rate_metric],
                }
            }
            Alterer::Crossover(name, rate, c) => {
                let (rate_value, rate_metric) = Self::rate_metric(generation, rate, name);

                let timer = std::time::Instant::now();
                let AlterResult(count, metrics, lineage_events) =
                    c.crossover(population, generation, rate_value);

                if let Some(lineage_events) = lineage_events {
                    lineage.extend(name, lineage_events);
                }

                let metric = metric!(name, (count, timer.elapsed()));

                match metrics {
                    Some(metrics) => metrics
                        .into_iter()
                        .chain(once(metric))
                        .chain(once(rate_metric))
                        .collect(),
                    None => vec![metric, rate_metric],
                }
            }
        }
    }

    #[inline]
    fn rate_metric(generation: usize, rate: &Rate, name: &str) -> (f32, Metric) {
        let rate_value = rate.value(generation);
        let metric = metric!(radiate_utils::intern!(format!("{}_rate", name)), rate_value);

        (rate_value, metric)
    }
}

/// Minimum population size required to perform crossover - this ensures that there
/// are enough individuals to select parents from. If the population size is
/// less than this value, we will not be able to select two distinct parents.
const MIN_POPULATION_SIZE: usize = 3;
/// Minimum number of parents required for crossover operation. This is typically
/// two, as crossover usually involves two parents to produce offspring.
const MIN_NUM_PARENTS: usize = 2;

thread_local! {
    static PARENT_SCRATCH_BUFFER: LazyCell<RefCell<Vec<usize>>> =LazyCell::new(|| RefCell::new(vec![0; MIN_NUM_PARENTS]));
}

/// The [Crossover] trait is used to define the crossover operation for a genetic algorithm.
///
/// In a genetic algorithm, crossover is a genetic operator used to vary the
/// programming of a chromosome or chromosomes from one generation to the next.
/// It is analogous to reproduction and biological crossover.
///
/// A [Crossover] typically takes two parent [Chromosome]s and produces two or more offspring [Chromosome]s.
/// This trait allows you to define your own crossover operation on either the entire population
/// or a subset of the population. If a struct implements the [Crossover] trait but does not override
/// any of the methods, the default implementation will perform a simple crossover operation on the
/// entire population.
pub trait Crossover<C: Chromosome>: Send + Sync {
    fn name(&self) -> String {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .map(|s| s.to_snake_case())
            .unwrap()
    }

    fn rate(&self) -> Rate {
        Rate::default()
    }

    fn alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::Crossover(intern!(self.name()), self.rate(), Arc::new(self))
    }

    #[inline]
    fn crossover(
        &self,
        population: &mut Population<C>,
        generation: usize,
        rate: f32,
    ) -> AlterResult {
        PARENT_SCRATCH_BUFFER.with(|buffer_cell| {
            let mut result = AlterResult::default();
            let mut buffer = buffer_cell.borrow_mut();
            for i in 0..population.len() {
                if random_provider::bool(rate) && population.len() > MIN_POPULATION_SIZE {
                    indexes::individual_indexes(i, population.len(), MIN_NUM_PARENTS, &mut buffer);
                    let cross_result = self.cross(population, &buffer, generation, rate);
                    result.merge(cross_result);
                }
            }

            result
        })
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

        if let Some((one, two)) = population.get_pair_mut(parent_indexes[0], parent_indexes[1]) {
            let mut cross_result = {
                let geno_one = one.genotype_mut();
                let geno_two = two.genotype_mut();

                let min_len = std::cmp::min(geno_one.len(), geno_two.len());
                let chromosome_index = random_provider::range(0..min_len);

                let chrom_one = &mut geno_one[chromosome_index];
                let chrom_two = &mut geno_two[chromosome_index];

                self.cross_chromosomes(chrom_one, chrom_two, rate)
            };

            if cross_result.count() > 0 {
                let parent_lineage = (one.family(), two.family());
                let parent_ids = (one.id(), two.id());
                one.invalidate(generation);
                two.invalidate(generation);

                cross_result.update_lineage((parent_lineage, parent_ids, one.id()));
                cross_result.update_lineage((parent_lineage, parent_ids, two.id()));
                result.merge(cross_result);
            }
        }

        result
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::bool(rate) {
                let gene_one = chrom_one.get(i);
                let gene_two = chrom_two.get(i);

                let new_gene_one = gene_one.with_allele(gene_two.allele());
                let new_gene_two = gene_two.with_allele(gene_one.allele());

                chrom_one.set(i, new_gene_one);
                chrom_two.set(i, new_gene_two);

                cross_count += 1;
            }
        }

        AlterResult::from(cross_count)
    }
}

pub trait Mutate<C: Chromosome>: Send + Sync {
    fn name(&self) -> String {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .map(|s| s.to_snake_case())
            .unwrap()
    }

    fn rate(&self) -> Rate {
        Rate::default()
    }

    fn alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::Mutate(intern!(self.name()), self.rate(), Arc::new(self))
    }

    #[inline]
    fn mutate(&self, population: &mut Population<C>, generation: usize, rate: f32) -> AlterResult {
        let mut result = AlterResult::default();

        for phenotype in population.iter_mut() {
            let mutate_result = self.mutate_genotype(phenotype.genotype_mut(), rate);

            if mutate_result.count() > 0 {
                let parent = (phenotype.family(), phenotype.id());
                phenotype.invalidate(generation);
                result.update_lineage((parent, phenotype.id()));
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
            if random_provider::bool(rate) {
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
