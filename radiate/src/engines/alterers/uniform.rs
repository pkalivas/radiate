use super::AlterAction;
use super::Alterer;
use super::Crossover;
use super::IntoAlter;
use super::Mutate;
use crate::Chromosome;

pub struct UniformCrossover {
    rate: f32,
}

impl UniformCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {}

impl<C: Chromosome> IntoAlter<C> for UniformCrossover {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "UniformCrossover",
            self.rate,
            AlterAction::Crossover(Box::new(self)),
        )
    }
}

pub struct UniformMutator {
    pub rate: f32,
}

impl UniformMutator {
    pub fn new(rate: f32) -> Self {
        UniformMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for UniformMutator {}

impl<C: Chromosome> IntoAlter<C> for UniformMutator {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "UniformMutator",
            self.rate,
            AlterAction::Mutate(Box::new(self)),
        )
    }
}
