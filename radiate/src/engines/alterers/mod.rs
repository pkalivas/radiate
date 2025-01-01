pub mod alter;
pub mod arithmetic;
pub mod bitflip;
pub mod gaussian;
pub mod intermediate;
pub mod invert;
pub mod mean;
pub mod multipoint;
pub mod pmx;
pub mod scramble;
pub mod shuffle;
pub mod simulated_binary;
pub mod swap;
pub mod uniform;

use std::{any::Any, vec};

pub use alter::*;
pub use arithmetic::*;
pub use bitflip::*;
pub use gaussian::*;
pub use intermediate::*;
pub use invert::*;
pub use mean::*;
pub use multipoint::*;
pub use pmx::*;
pub use scramble::*;
pub use shuffle::*;
pub use simulated_binary::*;
pub use swap::*;
pub use uniform::*;

use super::{random_provider, timer::Timer, Chromosome, Gene, Genotype, Metric, Population};

pub trait EngineComponent {
    fn name(&self) -> &'static str;
}

pub trait AltererTwo: EngineComponent {
    fn rate(&self) -> f32;
    fn alter_type(&self) -> AlterType;
}

pub trait Mutator<C: Chromosome>: AltererTwo {
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

        let mut new_metric = Metric::new_operations(self.name());
        new_metric.add_value(count as f32);
        new_metric.add_duration(timer.duration());

        vec![new_metric]
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

pub trait Crossover<C: Chromosome>: AltererTwo {
    fn alter(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric> {
        vec![]
    }
}

pub enum AlterWrapper<C: Chromosome> {
    Mutator(Box<dyn Mutator<C>>),
    Crossover(Box<dyn Crossover<C>>),
}

impl<C: Chromosome> AlterWrapper<C> {
    pub fn new_mutator(mutator: impl Mutator<C> + 'static) -> Self {
        AlterWrapper::Mutator(Box::new(mutator))
    }

    pub fn new_crossover(crossover: impl Crossover<C> + 'static) -> Self {
        AlterWrapper::Crossover(Box::new(crossover))
    }
}
