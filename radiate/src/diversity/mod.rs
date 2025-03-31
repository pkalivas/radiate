pub mod cosine;

pub use cosine::*;

use crate::{Chromosome, Phenotype};

pub trait DiversityMeasure<C: Chromosome>: Send + Sync {
    fn diversity(&self, one: &Phenotype<C>, two: &Phenotype<C>) -> f32;
}
