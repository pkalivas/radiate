use crate::Chromosome;
use crate::genome::phenotype::Phenotype;
use crate::objectives::Objective;
use radiate_utils::ToSnakeCase;
use std::fmt::Debug;

/// A trait for selection algorithms. Selection algorithms are used to select
/// individuals from a slice of [Phenotype]s to be used in the next generation.
/// The selection process is (most of the time) based on the fitness of the
/// individuals in the slice. Selectors return slice-relative indices —
/// callers map them back to whatever absolute coordinate space they came from.
pub trait Select<C: Chromosome>: Send + Sync + Debug {
    fn name(&self) -> &'static str {
        let name = radiate_utils::short_type_name::<Self>();
        let snake_case_name = name.to_snake_case();
        let other = snake_case_name
            .split('_')
            .rev()
            .collect::<Vec<_>>()
            .join(".");

        radiate_utils::intern!(other)
    }

    fn select(&self, population: &[Phenotype<C>], optimize: &Objective, count: usize)
    -> Vec<usize>;
}
