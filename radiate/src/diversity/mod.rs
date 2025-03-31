pub mod cosine;
use crate::{Chromosome, Phenotype};
pub use cosine::*;

pub trait DiversityMeasure<C: Chromosome>: Send + Sync {
    fn diversity(&self, one: &Phenotype<C>, two: &Phenotype<C>) -> f32;
}
