use crate::Chromosome;
use crate::genome::population::Population;
use crate::objectives::Objective;

/// A trait for selection algorithms. Selection algorithms are used to select
/// individuals from a [Population] to be used in the next generation. The
/// selection process is (most of the time) based on the fitness of the individuals in the
/// [Population]. The selection process can be based on the fitness of the individuals
/// in the [Population], or it can be based on the individuals themselves.
pub trait Select<C: Chromosome>: Send + Sync {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
            .split("<")
            .next()
            .unwrap_or(std::any::type_name::<Self>())
            .split("::")
            .last()
            .unwrap_or("Unknown Selector")
    }

    fn select(
        &self,
        population: &Population<C>,
        optimize: &Objective,
        count: usize,
    ) -> Population<C>;
}
