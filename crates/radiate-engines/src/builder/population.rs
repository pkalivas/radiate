use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Ecosystem, Population};

#[derive(Clone)]
pub struct PopulationParams<C: Chromosome> {
    pub population_size: usize,
    pub max_age: usize,
    pub ecosystem: Option<Ecosystem<C>>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Set the population size of the genetic engine. Default is 100.
    pub fn population_size(mut self, population_size: usize) -> Self {
        self.add_error_if(|| population_size < 3, "population_size must be at least 3");

        self.params.population_params.population_size = population_size;
        self
    }

    /// Set the maximum age of an individual in the population. Default is 25.
    pub fn max_age(mut self, max_age: usize) -> Self {
        self.add_error_if(|| max_age < 1, "max_age must be greater than 0");

        self.params.population_params.max_age = max_age;
        self
    }

    /// Set the population of the genetic engine. This is useful if you want to provide a custom population.
    /// If this is not set, the genetic engine will create a new population of `population_size` using the codec.
    pub fn population(mut self, population: impl Into<Population<C>>) -> Self {
        self.params.population_params.ecosystem = Some(Ecosystem::new(population.into()));
        self
    }

    pub fn ecosystem(mut self, ecosystem: impl Into<Ecosystem<C>>) -> Self {
        self.params.population_params.ecosystem = Some(ecosystem.into());
        self
    }
}
