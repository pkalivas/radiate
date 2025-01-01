use crate::alter::AlterType;
use crate::Alter;
use crate::Chromosome;

use super::AltererTwo;
use super::Crossover;
use super::EngineComponent;
use super::Mutator;

#[derive(Clone)]
pub struct UniformCrossover {
    pub rate: f32,
}

impl UniformCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<C: Chromosome> Alter<C> for UniformCrossover {
    fn name(&self) -> &'static str {
        "UniformCrossover"
    }
    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }
}

#[derive(Clone)]
pub struct UniformMutator {
    pub rate: f32,
}

impl UniformMutator {
    pub fn new(rate: f32) -> Self {
        UniformMutator { rate }
    }
}

impl<C: Chromosome> Alter<C> for UniformMutator {
    fn name(&self) -> &'static str {
        "UniformMutator"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Mutator
    }
}

impl EngineComponent for UniformMutator {
    fn name(&self) -> &'static str {
        "UniformMutator"
    }
}

impl AltererTwo for UniformMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }
}

impl<C: Chromosome> Mutator<C> for UniformMutator {}

impl<C: Chromosome> Crossover<C> for UniformCrossover {}

impl EngineComponent for UniformCrossover {
    fn name(&self) -> &'static str {
        "UniformCrossover"
    }
}

impl AltererTwo for UniformCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }
}
