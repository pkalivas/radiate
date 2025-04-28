use crate::{Chromosome, Genotype};

pub trait Diversity<C: Chromosome> {
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32;
}
