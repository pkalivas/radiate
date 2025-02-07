use super::{Crossover, Mutate};
use crate::{Chromosome, EngineCompoment};

pub enum AlterAction<C: Chromosome> {
    Mutate(Box<dyn Mutate<C>>),
    Crossover(Box<dyn Crossover<C>>),
}

pub trait Alter<C: Chromosome>: EngineCompoment {
    fn rate(&self) -> f32;
    fn to_alter(self) -> AlterAction<C>;
}
