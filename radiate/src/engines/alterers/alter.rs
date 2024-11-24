use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::population::Population;
use crate::engines::optimize::Optimize;
use crate::Metric;

use super::crossovers::crossover::Crossover;
use super::mutators::mutate::Mutate;

pub trait Alter<G, A>
where
    G: Gene<G, A>,
{
    fn alter(
        &self,
        population: &mut Population<G, A>,
        optimize: &Optimize,
        generation: i32,
    ) -> Vec<Metric>;
}

pub enum Alterer<G, A>
where
    G: Gene<G, A>,
{
    Mutator(f32),
    UniformCrossover(f32),
    MultiPointCrossover(f32, usize),
    SinglePointCrossover(f32),
    SwapMutator(f32),
    Mutation(Box<dyn Mutate<G, A>>),
    Crossover(Box<dyn Crossover<G, A>>),
    Alterer(Box<dyn Alter<G, A>>),
}

impl<G, A> Alterer<G, A>
where
    G: Gene<G, A>,
{
    pub fn alterer<T: Alter<G, A> + 'static>(alterer: T) -> Self {
        Alterer::Alterer(Box::new(alterer))
    }

    pub fn crossover<T: Crossover<G, A> + 'static>(crossover: T) -> Self {
        Alterer::Crossover(Box::new(crossover))
    }

    pub fn mutation<T: Mutate<G, A> + 'static>(mutation: T) -> Self {
        Alterer::Mutation(Box::new(mutation))
    }
}

pub struct AlterWrap<G, A>
where
    G: Gene<G, A>,
{
    pub rate: f32,
    pub mutator: Option<Box<dyn Mutate<G, A>>>,
    pub crossover: Option<Box<dyn Crossover<G, A>>>,
    pub alterer: Option<Box<dyn Alter<G, A>>>,
}

impl<G: Gene<G, A>, A> AlterWrap<G, A> {
    pub fn from_mutator(mutator: Box<dyn Mutate<G, A>>, rate: f32) -> Self {
        Self {
            rate,
            mutator: Some(mutator),
            crossover: None,
            alterer: None,
        }
    }

    pub fn from_crossover(crossover: Box<dyn Crossover<G, A>>, rate: f32) -> Self {
        Self {
            rate,
            mutator: None,
            crossover: Some(crossover),
            alterer: None,
        }
    }

    pub fn from_alterer(alterer: Box<dyn Alter<G, A>>, rate: f32) -> Self {
        Self {
            rate,
            mutator: None,
            crossover: None,
            alterer: Some(alterer),
        }
    }
}
