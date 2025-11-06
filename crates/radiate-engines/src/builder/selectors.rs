use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Select};
use std::sync::Arc;

#[derive(Clone)]
pub struct SelectionParams<C: Chromosome> {
    pub offspring_fraction: f32,
    pub survivor_selector: Arc<dyn Select<C>>,
    pub offspring_selector: Arc<dyn Select<C>>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Set the fraction of the population that will be replaced by offspring each generation.
    /// Default is 0.8. This is a value from 0...=1 that represents the fraction of
    /// population that will be replaced by offspring each generation. The remainder will 'survive' to the next generation.
    pub fn offspring_fraction(mut self, offspring_fraction: f32) -> Self {
        self.add_error_if(
            || !(0.0..=1.0).contains(&offspring_fraction),
            "offspring_fraction must be between 0.0 and 1.0",
        );

        self.params.selection_params.offspring_fraction = offspring_fraction;
        self
    }

    pub fn boxed_survivor_selector(mut self, selector: Box<dyn Select<C>>) -> Self {
        self.params.selection_params.survivor_selector = selector.into();
        self
    }

    pub fn boxed_offspring_selector(mut self, selector: Box<dyn Select<C>>) -> Self {
        self.params.selection_params.offspring_selector = selector.into();
        self
    }

    /// Set the survivor selector of the genetic engine. This is the selector that will
    /// be used to select the survivors of the population. Default is `TournamentSelector`
    /// with a group size of 3.
    pub fn survivor_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.params.selection_params.survivor_selector = Arc::new(selector);
        self
    }

    /// Set the offspring selector of the genetic engine. This is the selector that will
    /// be used to select the offspring of the population. Default is `RouletteSelector`.
    pub fn offspring_selector<S: Select<C> + 'static>(mut self, selector: S) -> Self {
        self.params.selection_params.offspring_selector = Arc::new(selector);
        self
    }
}
