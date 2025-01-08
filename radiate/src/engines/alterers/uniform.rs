use crate::Chromosome;

use super::Alter;
use super::AlterAction;
use super::Crossover;
use super::EngineCompoment;
use super::Mutate;

pub struct UniformCrossover {
    rate: f32,
}

impl UniformCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<C: Chromosome> Alter<C> for UniformCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl EngineCompoment for UniformCrossover {
    fn name(&self) -> &'static str {
        "UniformCrossover"
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {}

pub struct UniformMutator {
    pub rate: f32,
}

impl UniformMutator {
    pub fn new(rate: f32) -> Self {
        UniformMutator { rate }
    }
}

impl EngineCompoment for UniformMutator {
    fn name(&self) -> &'static str {
        "UniformMutator"
    }
}

impl<C: Chromosome> Alter<C> for UniformMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome> Mutate<C> for UniformMutator {}
