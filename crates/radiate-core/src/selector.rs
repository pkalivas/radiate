use crate::Chromosome;
use crate::genome::population::Population;
use crate::objectives::Objective;
use radiate_utils::ToSnakeCase;
use std::fmt::Debug;

/// A trait for selection algorithms. Selection algorithms are used to select
/// individuals from a [Population] to be used in the next generation. The
/// selection process is (most of the time) based on the fitness of the individuals in the
/// [Population]. The selection process can be based on the fitness of the individuals
/// in the [Population], or it can be based on the individuals themselves.
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

    fn select(&self, population: &Population<C>, optimize: &Objective, count: usize) -> Vec<usize>;

    // fn select_idx(
    //     &self,
    //     population: &Population<C>,
    //     optimize: &Objective,
    //     count: usize,
    // ) -> Vec<usize> {
    //     panic!("select_idx is not implemented for {}", self.name());
    // }
}
