use crate::{Chromosome, Gene, Genotype, math::indexes, random_provider};
use crate::{GetPairMut, MetricSet, Phenotype};
use crate::{RateSet, error::RadiateResult};
pub use radiate_expr::*;
use radiate_utils::{SmallStr, ToSnakeCase, intern};
use std::collections::HashMap;
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
#[derive(Default)]
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

impl From<usize> for AlterResult {
    fn from(value: usize) -> Self {
        AlterResult(value)
    }
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
    pub fn mutation(name: &'static str, m: Arc<dyn Mutate<C>>) -> Self {
        Self::build_internal(name, AlterInner::Mutate(m))
    }

    pub fn crossover(name: &'static str, c: Arc<dyn Crossover<C>>) -> Self {
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
                let timer = std::time::Instant::now();
                let mutator = Arc::get_mut(&mut (*m));

                if let Some(mutator) = mutator {
                    let result = mutator.mutate(population, &mut ctx);
                    metrics.upsert(&self.name, result.count());
                    metrics.upsert(&self.time_name, timer.elapsed());

                    for (name, count) in ctx.alter_counts.iter() {
                        metrics.upsert(name, *count);
                    }
                }
            }
            AlterInner::Crossover(c) => {
                let timer = std::time::Instant::now();
                let result = c.crossover(population, &mut ctx);
                metrics.upsert(&self.name, result.count());
                metrics.upsert(&self.time_name, timer.elapsed());

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
        let name = std::any::type_name::<Self>()
            .split("::")
            .last()
            .map(|s| s.to_snake_case())
            .unwrap();

        let path = name.split('_').collect::<Vec<&str>>();
        let mut new_name = vec!["crossover"];
        for part in path {
            if !part.contains("crossov") {
                new_name.push(part);
            }
        }

        new_name.join(".")
    }

    fn rates(&self) -> RateSet {
        RateSet::default()
    }

    fn alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::crossover(intern!(self.name()), Arc::new(self))
    }

    #[inline]
    fn crossover(&self, population: &mut [Phenotype<C>], ctx: &mut AlterContext) -> AlterResult {
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
        mut population: &mut [Phenotype<C>],
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
                one.invalidate(ctx.generation());
                two.invalidate(ctx.generation());

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
                let gene_one = chrom_one.get_mut(i);
                let gene_two = chrom_two.get_mut(i);

                if let Some((gene_one, gene_two)) = gene_one.zip(gene_two) {
                    // let new_gene_one = gene_one.with_allele(gene_two.allele());
                    // let new_gene_two = gene_two.with_allele(gene_one.allele());

                    // chrom_one.set(i, gene_two);
                    // chrom_two.set(i, gene_one);

                    std::mem::swap(gene_one, gene_two);

                    cross_count += 1;
                }
            }
        }

        AlterResult::from(cross_count)
    }
}

pub trait Mutate<C: Chromosome>: Send + Sync {
    fn name(&self) -> String {
        let name = std::any::type_name::<Self>()
            .split("::")
            .last()
            .map(|s| s.to_snake_case())
            .unwrap();

        let path = name.split('_').collect::<Vec<&str>>();
        let mut new_name = vec!["mutator"];
        for part in path {
            if !part.contains("mutat") {
                new_name.push(part);
            }
        }

        new_name.join(".")
    }

    fn rates(&self) -> RateSet {
        RateSet::default()
    }

    fn alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::mutation(intern!(self.name()), Arc::new(self))
    }

    #[inline]
    fn mutate(&mut self, population: &mut [Phenotype<C>], ctx: &mut AlterContext) -> AlterResult {
        let mut result = AlterResult::default();

        for phenotype in population.iter_mut() {
            let mutate_result = self.mutate_genotype(phenotype.genotype_mut(), ctx);

            if mutate_result.count() > 0 {
                phenotype.invalidate(ctx.generation());
            }

            result.merge(mutate_result);
        }

        result
    }

    #[inline]
    fn mutate_genotype(
        &mut self,
        genotype: &mut Genotype<C>,
        ctx: &mut AlterContext,
    ) -> AlterResult {
        let mut result = AlterResult::default();

        for chromosome in genotype.iter_mut() {
            let mutate_result = self.mutate_chromosome(chromosome, ctx);
            result.merge(mutate_result);
        }

        result
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> AlterResult {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::bool(ctx.rate()) {
                // count += self.mutate_gene(gene);
                panic!("mutate_gene is not implemented for this mutator");
            }
        }

        count.into()
    }

    // #[inline]
    // fn mutate_gene(&self, gene: &mut C::Gene) -> usize {
    //     *gene = gene.new_instance();
    //     1
    // }
}

// fn calculate_rates(&mut self, metrics: &MetricSet) -> RadiateResult<()> {
//     self.rates.clear();
//     for expr in self.expr_set.iter_mut() {
//         if let Some(rate) = metrics.get(expr.name()).map(|v| v.last_value()) {
//             self.rates.push(rate);
//         } else {
//             let rate = match expr.eval(metrics)?.extract::<f32>() {
//                 Some(rate) => rate,
//                 None => {
//                     radiate_bail!(Expr:
//                         "Failed to evaluate rate expression for alterer {}: {}",
//                         self.name,
//                         expr.name()
//                     );
//                 }
//             };

//             self.rates.push(rate);
//         }
//     }

//     Ok(())
// }
