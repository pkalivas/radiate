use crate::{Chromosome, Genotype};

pub trait Distance<C: Chromosome>: Send + Sync {
    fn threshold(&self) -> f32;
    fn distance(&self, a: &Genotype<C>, b: &Genotype<C>) -> f32;
}
