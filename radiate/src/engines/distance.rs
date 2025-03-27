use super::{Chromosome, Genotype};

pub trait Distance<C: Chromosome> {
    fn distance(&self, a: &Genotype<C>, b: &Genotype<C>) -> f32;
}
