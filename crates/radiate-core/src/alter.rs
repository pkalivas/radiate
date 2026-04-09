use crate::{Chromosome, Gene, Genotype, Population, math::indexes, random_provider};
use crate::{Lineage, LineageUpdate, MetricSet, MetricUpdate, Rate, metric};
use radiate_utils::{ToSnakeCase, intern};
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
pub struct AlterResult(pub usize);

impl AlterResult {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn count(&self) -> usize {
        self.0
    }

    pub fn merge(&mut self, other: AlterResult) {
        let AlterResult(other_count) = other;
        self.0 += other_count;
    }
}

impl Default for AlterResult {
    fn default() -> Self {
        AlterResult(0)
    }
}

impl From<usize> for AlterResult {
    fn from(value: usize) -> Self {
        AlterResult(value)
    }
}

pub struct AlterContext<'a> {
    metrics: &'a mut MetricSet,
    lineage: &'a mut Lineage,
    operation: &'static str,
    generation: usize,
    rate: f32,
}

impl<'a> AlterContext<'a> {
    pub fn new(
        operation: &'static str,
        metrics: &'a mut MetricSet,
        lineage: &'a mut Lineage,
        generation: usize,
        rate: f32,
    ) -> Self {
        Self {
            metrics,
            lineage,
            operation,
            generation,
            rate,
        }
    }

    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn metric(&mut self, name: &'static str, value: impl Into<MetricUpdate<'a>>) {
        self.metrics.upsert((name, value.into()));
    }

    pub fn update_lineage(&mut self, update: impl Into<LineageUpdate>) {
        self.lineage.push(self.operation, update.into());
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
    pub fn name(&self) -> &'static str {
        match &self {
            Alterer::Mutate(name, _, _) => name,
            Alterer::Crossover(name, _, _) => name,
        }
    }

    pub fn rate(&mut self) -> &mut Rate {
        match self {
            Alterer::Mutate(_, rate, _) => rate,
            Alterer::Crossover(_, rate, _) => rate,
        }
    }

    #[inline]
    pub fn alter(
        &mut self,
        population: &mut Population<C>,
        lineage: &mut Lineage,
        metrics: &mut MetricSet,
        generation: usize,
    ) {
        let rate = self.rate().value_from_metrics(metrics);
        let operation = self.name();

        metrics.upsert(metric!(
            radiate_utils::intern!(format!("{}_rate", operation)),
            rate
        ));

        let mut ctx = AlterContext {
            metrics,
            lineage,
            operation,
            generation,
            rate,
        };

        match &self {
            Alterer::Mutate(name, _, m) => {
                let timer = std::time::Instant::now();
                let AlterResult(count) = m.mutate(population, &mut ctx);
                metrics.upsert(metric!(name, (count, timer.elapsed())));
            }
            Alterer::Crossover(name, _, c) => {
                let timer = std::time::Instant::now();
                let AlterResult(count) = c.crossover(population, &mut ctx);
                metrics.upsert(metric!(name, (count, timer.elapsed())));
            }
        }
    }
}

/// Minimum population size required to perform crossover - this ensures that there
/// are enough individuals to select parents from. If the population size is
/// less than this value, we will not be able to select two distinct parents.
const MIN_POPULATION_SIZE: usize = 3;
/// Minimum number of parents required for crossover operation. This is typically
/// two, as crossover usually involves two parents to produce offspring.
const MIN_NUM_PARENTS: usize = 2;

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
    fn crossover(&self, population: &mut Population<C>, ctx: &mut AlterContext) -> AlterResult {
        let mut result = AlterResult::default();
        let mut parents = [0usize; MIN_NUM_PARENTS];

        for i in 0..population.len() {
            if random_provider::bool(ctx.rate()) && population.len() > MIN_POPULATION_SIZE {
                indexes::individual_indexes(i, population.len(), MIN_NUM_PARENTS, &mut parents);
                let cross_result = self.cross(population, &parents, ctx);
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
        ctx: &mut AlterContext,
    ) -> AlterResult {
        let mut result = AlterResult::default();

        if let Some((one, two)) = population.get_pair_mut(parent_indexes[0], parent_indexes[1]) {
            let cross_result = {
                let geno_one = one.genotype_mut();
                let geno_two = two.genotype_mut();

                let min_len = std::cmp::min(geno_one.len(), geno_two.len());
                let chromosome_index = random_provider::range(0..min_len);

                let chrom_one = &mut geno_one[chromosome_index];
                let chrom_two = &mut geno_two[chromosome_index];

                self.cross_chromosomes(chrom_one, chrom_two, ctx)
            };

            if cross_result.count() > 0 {
                let parent_lineage = (one.family(), two.family());
                let parent_ids = (one.id(), two.id());
                one.invalidate(ctx.generation());
                two.invalidate(ctx.generation());

                ctx.update_lineage((parent_lineage, parent_ids, one.id()));
                ctx.update_lineage((parent_lineage, parent_ids, two.id()));
                result.merge(cross_result);
            }
        }

        result
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut C,
        chrom_two: &mut C,
        ctx: &mut AlterContext,
    ) -> AlterResult {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::bool(ctx.rate()) {
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
    fn mutate(&self, population: &mut Population<C>, ctx: &mut AlterContext) -> AlterResult {
        let mut result = AlterResult::default();

        for phenotype in population.iter_mut() {
            let mutate_result = self.mutate_genotype(phenotype.genotype_mut(), ctx);

            if mutate_result.count() > 0 {
                let parent = (phenotype.family(), phenotype.id());
                phenotype.invalidate(ctx.generation());
                ctx.update_lineage((parent, phenotype.id()));
            }

            result.merge(mutate_result);
        }

        result
    }

    #[inline]
    fn mutate_genotype(&self, genotype: &mut Genotype<C>, ctx: &mut AlterContext) -> AlterResult {
        let mut result = AlterResult::default();

        for chromosome in genotype.iter_mut() {
            let mutate_result = self.mutate_chromosome(chromosome, ctx);
            result.merge(mutate_result);
        }

        result
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, ctx: &mut AlterContext) -> AlterResult {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::bool(ctx.rate()) {
                count += self.mutate_gene(gene);
            }
        }

        count.into()
    }

    #[inline]
    fn mutate_gene(&self, gene: &mut C::Gene) -> usize {
        *gene = gene.new_instance();
        1
    }
}
