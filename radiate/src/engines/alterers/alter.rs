use crate::engines::genome::population::Population;
use crate::engines::optimize::Optimize;
use crate::{Chromosome, Metric};

use super::crossovers::crossover::Crossover;
use super::mutators::mutate::Mutate;

pub trait Alter<C: Chromosome> {
    fn alter(
        &self,
        population: &mut Population<C>,
        optimize: &Optimize,
        generation: i32,
    ) -> Vec<Metric>;
}

pub enum Alterer<C: Chromosome> {
    Mutator(f32),
    UniformCrossover(f32),
    MultiPointCrossover(f32, usize),
    SinglePointCrossover(f32),
    SwapMutator(f32),
    Mutation(Box<dyn Mutate<C>>),
    Crossover(Box<dyn Crossover<C>>),
    Alterer(Box<dyn Alter<C>>),
}

impl<C: Chromosome> Alterer<C> {
    pub fn alterer<T: Alter<C> + 'static>(alterer: T) -> Self {
        Alterer::Alterer(Box::new(alterer))
    }

    pub fn crossover<T: Crossover<C> + 'static>(crossover: T) -> Self {
        Alterer::Crossover(Box::new(crossover))
    }

    pub fn mutation<T: Mutate<C> + 'static>(mutation: T) -> Self {
        Alterer::Mutation(Box::new(mutation))
    }
}

pub struct AlterWrap<C: Chromosome> {
    pub rate: f32,
    pub mutator: Option<Box<dyn Mutate<C>>>,
    pub crossover: Option<Box<dyn Crossover<C>>>,
    pub alterer: Option<Box<dyn Alter<C>>>,
}

impl<C: Chromosome> AlterWrap<C> {
    pub fn from_mutator(mutator: Box<dyn Mutate<C>>, rate: f32) -> Self {
        Self {
            rate,
            mutator: Some(mutator),
            crossover: None,
            alterer: None,
        }
    }

    pub fn from_crossover(crossover: Box<dyn Crossover<C>>, rate: f32) -> Self {
        Self {
            rate,
            mutator: None,
            crossover: Some(crossover),
            alterer: None,
        }
    }

    pub fn from_alterer(alterer: Box<dyn Alter<C>>, rate: f32) -> Self {
        Self {
            rate,
            mutator: None,
            crossover: None,
            alterer: Some(alterer),
        }
    }
}
