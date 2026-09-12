use crate::{Chromosome, Gene, MetricSet, math::indexes, random_provider, stats::metric_tags};
use crate::{GetPairMut, Phenotype};
use crate::{RateSet, error::RadiateResult};
pub use radiate_expr::*;
use radiate_utils::{SmallStr, generate_metric_key};
use std::collections::HashMap;
use std::sync::Arc;

#[macro_export]
macro_rules! alters {
    ($($struct_instance:expr),* $(,)?) => {
        {
            let mut vec: Vec<Alterer<_>> = Vec::new();
            $(
                vec.push($struct_instance.into_alterer());
            )*
            vec
        }
    };
}

#[derive(Clone, Default)]
pub struct AlterUpdates(pub HashMap<SmallStr, usize>);

impl AlterUpdates {
    pub fn new() -> Self {
        AlterUpdates(HashMap::new())
    }

    pub fn clear(&mut self) {
        for value in self.0.values_mut() {
            *value = 0;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SmallStr, &usize)> {
        self.0.iter().filter(|(_, count)| **count > 0)
    }

    pub fn upsert(&mut self, name: impl AsRef<str>, value: usize) {
        if let Some(existing) = self.0.get_mut(name.as_ref()) {
            *existing += value;
        } else {
            self.0
                .insert(SmallStr::from_string(name.as_ref().into()), value);
        }
    }
}

pub struct AlterContext<'a> {
    alter_counts: &'a mut AlterUpdates,
    generation: usize,
    control_rate: f32,
    internal_rates: &'a [f32],
}

impl<'a> AlterContext<'a> {
    pub fn new(
        alter_counts: &'a mut AlterUpdates,
        generation: usize,
        control_rate: f32,
        internal_rates: &'a [f32],
    ) -> Self {
        AlterContext {
            alter_counts,
            generation,
            control_rate,
            internal_rates,
        }
    }

    pub fn rate(&self) -> f32 {
        self.control_rate
    }

    pub fn internal_rate(&self, index: usize) -> f32 {
        self.internal_rates.get(index).copied().unwrap_or(0.0)
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn upsert(&mut self, name: impl AsRef<str>, value: usize) {
        self.alter_counts.upsert(name, value);
    }
}

#[derive(Clone)]
pub enum AlterInner<C: Chromosome> {
    Mutate(Arc<dyn Mutate<C>>),
    Crossover(Arc<dyn Crossover<C>>),
}

/// The [Alterer] struct is used to represent the different
/// types of alterations that can be performed on a
/// population - It can be either a mutation or a crossover operation.
#[derive(Clone)]
pub struct Alterer<C: Chromosome> {
    time_name: SmallStr,
    name: SmallStr,
    inner: AlterInner<C>,
    alter_counts: AlterUpdates,
    rate_set: RateSet,
}

impl<C: Chromosome> Alterer<C> {
    pub fn mutation(name: impl Into<SmallStr>, m: Arc<dyn Mutate<C>>) -> Self {
        Self::build_internal(name, AlterInner::Mutate(m))
    }

    pub fn crossover(name: impl Into<SmallStr>, c: Arc<dyn Crossover<C>>) -> Self {
        Self::build_internal(name, AlterInner::Crossover(c))
    }

    fn build_internal(name: impl Into<SmallStr>, inner: AlterInner<C>) -> Self {
        let name = name.into();

        let time_name = SmallStr::from_string(format!("{}.time", name));
        let control_rate_name = SmallStr::from_string(format!("{}.rate", name));

        let rate_set = match &inner {
            AlterInner::Mutate(m) => m.rates().alias(control_rate_name.clone()),
            AlterInner::Crossover(c) => c.rates().alias(control_rate_name.clone()),
        };

        Self {
            time_name,
            name,
            inner,
            alter_counts: AlterUpdates::new(),
            rate_set,
        }
    }

    pub fn rates(&self) -> &RateSet {
        &self.rate_set
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn alter(
        &mut self,
        population: &mut [Phenotype<C>],
        metrics: &mut MetricSet,
        generation: usize,
    ) -> RadiateResult<()> {
        let rates = self.rate_set.calculate_rates(generation, metrics)?;

        self.alter_counts.clear();

        let mut ctx = AlterContext {
            alter_counts: &mut self.alter_counts,
            generation,
            control_rate: rates[0],
            internal_rates: &rates[1..],
        };

        match &mut self.inner {
            AlterInner::Mutate(m) => {
                let mutator = Arc::get_mut(&mut (*m)).unwrap();

                let timer = std::time::Instant::now();
                let result = mutator.mutate(population, &mut ctx);
                metrics.upsert(&self.time_name, timer.elapsed());
                metrics.upsert(&self.name, result);

                for (name, count) in ctx.alter_counts.iter() {
                    metrics.upsert(name, *count);
                }
            }
            AlterInner::Crossover(c) => {
                let timer = std::time::Instant::now();
                let result = c.crossover(population, &mut ctx);
                metrics.upsert(&self.time_name, timer.elapsed());
                metrics.upsert(&self.name, result);

                for (name, count) in ctx.alter_counts.iter() {
                    metrics.upsert(name, *count);
                }
            }
        }

        Ok(())
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
        generate_metric_key::<Self>(metric_tags::CROSSOVER)
    }

    fn into_alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::crossover(self.name(), Arc::new(self))
    }

    fn rates(&self) -> RateSet {
        RateSet::default()
    }

    #[inline]
    fn crossover(&self, mut population: &mut [Phenotype<C>], ctx: &mut AlterContext) -> usize {
        let mut result = 0;
        let mut parents = [0; MIN_NUM_PARENTS];
        let pop_size = population.len();

        for i in 0..pop_size {
            if random_provider::bool(ctx.rate()) && pop_size > MIN_POPULATION_SIZE {
                indexes::fill_subset_inclusive(i, pop_size, &mut parents);

                result += population
                    .get_pair_mut(parents[0], parents[1])
                    .map(|(one, two)| self.cross(one, two, ctx))
                    .unwrap_or(0);
            }
        }

        result
    }

    #[inline]
    fn cross(
        &self,
        parent_one: &mut Phenotype<C>,
        parent_two: &mut Phenotype<C>,
        ctx: &mut AlterContext,
    ) -> usize {
        let geno_one = parent_one.genotype_mut();
        let geno_two = parent_two.genotype_mut();

        let min_len = std::cmp::min(geno_one.len(), geno_two.len());
        let chromosome_index = random_provider::range(0..min_len);

        let chrom_one = &mut geno_one[chromosome_index];
        let chrom_two = &mut geno_two[chromosome_index];

        let cross_result = self.cross_chromosomes(chrom_one, chrom_two, ctx);

        if cross_result > 0 {
            parent_one.invalidate(ctx.generation());
            parent_two.invalidate(ctx.generation());
        }

        cross_result
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut C,
        chrom_two: &mut C,
        ctx: &mut AlterContext,
    ) -> usize {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::bool(ctx.rate()) {
                let gene_one = chrom_one.get_mut(i);
                let gene_two = chrom_two.get_mut(i);

                if let Some((gene_one, gene_two)) = gene_one.zip(gene_two) {
                    std::mem::swap(gene_one, gene_two);
                    cross_count += 1;
                }
            }
        }

        cross_count
    }
}

pub trait Mutate<C: Chromosome>: Send + Sync {
    fn name(&self) -> String {
        generate_metric_key::<Self>(metric_tags::MUTATOR)
    }

    fn into_alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::mutation(self.name(), Arc::new(self))
    }

    fn rates(&self) -> RateSet {
        RateSet::default()
    }

    #[inline]
    fn mutate(&mut self, population: &mut [Phenotype<C>], ctx: &mut AlterContext) -> usize {
        population
            .iter_mut()
            .map(|phenotype| {
                let mutate_result = phenotype
                    .genotype_mut()
                    .iter_mut()
                    .fold(0, |acc, chromosome| {
                        acc + self.mutate_chromosome(chromosome, ctx)
                    });

                if mutate_result > 0 {
                    phenotype.invalidate(ctx.generation());
                }

                mutate_result
            })
            .sum()
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        chromosome
            .iter_mut()
            .filter(|_| random_provider::bool(ctx.rate()))
            .fold(0, |acc, gene| {
                *gene = gene.new_instance();
                acc + 1
            })
    }
}
