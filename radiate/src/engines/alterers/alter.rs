use crate::{Chromosome, EngineCompoment};

use super::{Crossover, Mutate};

pub enum AlterAction<C: Chromosome> {
    Mutate(Box<dyn Mutate<C>>),
    Crossover(Box<dyn Crossover<C>>),
}

pub trait Alter<C: Chromosome>: EngineCompoment {
    fn rate(&self) -> f32;
    fn to_alter(self) -> AlterAction<C>;
}
