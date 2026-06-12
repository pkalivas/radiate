use crate::{Chromosome, Gene, Genotype, math::indexes, random_provider};
use crate::{GetPairMut, MetricSet, Phenotype, Rate};
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
    rate: f32,
}

impl<'a> AlterContext<'a> {
    pub fn new(alter_counts: &'a mut AlterUpdates, generation: usize, rate: f32) -> Self {
        AlterContext {
            alter_counts,
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
    name: SmallStr,
    time_name: SmallStr,
    rate_name: SmallStr,
    rate: Rate,
    inner: AlterInner<C>,
    alter_counts: AlterUpdates,
}

impl<C: Chromosome> Alterer<C> {
    pub fn mutation(name: &'static str, rate: Rate, m: Arc<dyn Mutate<C>>) -> Self {
        let name = SmallStr::from_static(name);
        Self {
            time_name: SmallStr::from_string(format!("{}.time", name)),
            rate_name: SmallStr::from_string(format!("{}.rate", name)),
            name,
            rate,
            inner: AlterInner::Mutate(m),
            alter_counts: AlterUpdates::new(),
        }
    }

    pub fn crossover(name: &'static str, rate: Rate, c: Arc<dyn Crossover<C>>) -> Self {
        let name = SmallStr::from_static(name);
        Self {
            time_name: SmallStr::from_string(format!("{}.time", name)),
            rate_name: SmallStr::from_string(format!("{}.rate", name)),
            name,
            rate,
            inner: AlterInner::Crossover(c),
            alter_counts: AlterUpdates::new(),
        }
    }

    pub fn rate(&self) -> &Rate {
        &self.rate
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn alter(
        &mut self,
        population: &mut [Phenotype<C>],
        metrics: &mut MetricSet,
        generation: usize,
    ) {
        let rate = self.rate.get(generation, metrics);

        metrics.upsert(self.rate_name.clone(), rate);
        self.alter_counts.clear();

        let mut ctx = AlterContext {
            alter_counts: &mut self.alter_counts,
            generation,
            rate,
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

    fn rate(&self) -> Rate {
        Rate::default()
    }

    fn alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::crossover(intern!(self.name()), self.rate(), Arc::new(self))
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

    fn rate(&self) -> Rate {
        Rate::default()
    }

    fn alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        Alterer::mutation(intern!(self.name()), self.rate(), Arc::new(self))
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
