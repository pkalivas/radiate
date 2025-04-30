use crate::{Chromosome, Genotype};

pub trait Diversity<C: Chromosome>: Send + Sync {
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32;
}
