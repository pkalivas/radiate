use crate::alter::AlterType;
use crate::Alter;
use crate::Chromosome;

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
