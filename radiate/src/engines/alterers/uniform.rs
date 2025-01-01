use crate::Chromosome;

use super::AlterAction;
use super::CrossoverAction;
use super::EngineAlterer;
use super::EngineCompoment;
use super::MutateAction;

pub struct UniformCrossover {
    pub rate: f32,
}

impl UniformCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
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

impl EngineCompoment for UniformMutator {
    fn name(&self) -> &'static str {
        "UniformMutator"
    }
}

impl<C: Chromosome> EngineAlterer<C> for UniformMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome> MutateAction<C> for UniformMutator {}

impl<C: Chromosome> EngineAlterer<C> for UniformCrossover {
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

impl<C: Chromosome> CrossoverAction<C> for UniformCrossover {}
