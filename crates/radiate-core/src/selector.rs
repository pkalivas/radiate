use crate::genome::phenotype::Phenotype;
use crate::objectives::Objective;
use crate::{Chromosome, stats::metric_tags};
use radiate_utils::generate_metric_key;

/// A trait for selection algorithms. Selection algorithms are used to select
/// individuals from a slice of [Phenotype]s to be used in the next generation.
/// The selection process is (most of the time) based on the fitness of the
/// individuals in the slice. Selectors return slice-relative indices —
/// callers map them back to whatever absolute coordinate space they came from.
pub trait Select<C: Chromosome>: Send + Sync {
    fn name(&self) -> String {
        generate_metric_key::<Self>(metric_tags::SELECTOR)
    }

    fn select(&self, population: &[Phenotype<C>], optimize: &Objective, count: usize)
    -> Vec<usize>;
}
