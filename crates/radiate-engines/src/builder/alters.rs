use crate::GeneticEngineBuilder;
use radiate_core::{Alter, AlterAction, Chromosome, Crossover, Mutate};
use std::sync::Arc;

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Set the alterer of the genetic engine. This is the alterer that will be used to
    /// alter the offspring of the population. The alterer is used to apply mutations
    /// and crossover operations to the offspring and will be used to create the next
    /// generation of the population. **Note**: the order of the alterers is important - the
    /// alterers will be applied in the order they are provided.
    pub fn alter(mut self, alterers: Vec<Box<dyn Alter<C>>>) -> Self {
        self.params.alterers = alterers.into_iter().map(|alt| alt.into()).collect();
        self
    }

    /// Define a single mutator for the genetic engine - this will be  converted to
    /// a `Box<dyn Alter<C>>` and added to the list of alterers. Note: The order in which
    /// mutators and crossovers are added is the order in which they will be applied during
    /// the evolution process.
    pub fn mutator<M: Mutate<C> + 'static>(mut self, mutator: M) -> Self {
        self.params.alterers.push(Arc::new(mutator.alterer()));
        self
    }

    /// Define a list of mutators for the genetic engine - this will be converted to a list
    /// of `Box<dyn Alter<C>>` and added to the list of alterers. Just like adding a single mutator,
    /// the order in which mutators and crossovers are added is the order in which they will be applied
    /// during the evolution process.s
    pub fn mutators(mut self, mutators: Vec<Box<dyn Mutate<C>>>) -> Self {
        let mutate_actions = mutators
            .into_iter()
            .map(|m| {
                Arc::new(AlterAction::Mutate(m.name().leak(), m.rate(), m)) as Arc<dyn Alter<C>>
            })
            .collect::<Vec<_>>();

        self.params.alterers.extend(mutate_actions);
        self
    }

    /// Define a single crossover for the genetic engine - this will be converted to
    /// a `Box<dyn Alter<C>>` and added to the list of alterers. Note: The order in which
    /// mutators and crossovers are added is the order in which they will be applied during
    /// the evolution process.s
    pub fn crossover<R: Crossover<C> + 'static>(mut self, crossover: R) -> Self {
        self.params.alterers.push(Arc::new(crossover.alterer()));
        self
    }

    /// Define a list of crossovers for the genetic engine - this will be converted to a list
    /// of `Box<dyn Alter<C>>` and added to the list of alterers. Just like adding a single crossover,
    /// the order in which mutators and crossovers are added is the order in which they will be applied
    /// during the evolution process.
    pub fn crossovers(mut self, crossovers: Vec<Box<dyn Crossover<C>>>) -> Self {
        let crossover_actions = crossovers
            .into_iter()
            .map(|c| {
                Arc::new(AlterAction::Crossover(c.name().leak(), c.rate(), c)) as Arc<dyn Alter<C>>
            })
            .collect::<Vec<_>>();

        self.params.alterers.extend(crossover_actions);
        self
    }
}
