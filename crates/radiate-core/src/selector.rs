use crate::Chromosome;
use crate::genome::population::Population;
use crate::objectives::Objective;
use crate::parameter::Parameter;

/// A trait for selection algorithms. Selection algorithms are used to select
/// individuals from a [Population] to be used in the next generation. The
/// selection process is (most of the time) based on the fitness of the individuals in the
/// [Population]. The selection process can be based on the fitness of the individuals
/// in the [Population], or it can be based on the individuals themselves.
pub trait Select<C: Chromosome>: Send + Sync {
    fn name(&self) -> &'static str {
        let name = std::any::type_name::<Self>()
            .split("<")
            .next()
            .unwrap_or(std::any::type_name::<Self>())
            .split("::")
            .last()
            .unwrap_or("Unknown Selector");

        if let Some(interned) = radiate_utils::try_get_interned_str(name) {
            return interned;
        }

        let snake_case_name = radiate_utils::intern_name_as_snake_case(name);

        let mut parts = snake_case_name
            .split('_')
            .filter(|part| !part.is_empty() && !part.contains("select"))
            .collect::<Vec<_>>();

        parts.insert(0, "selector");

        radiate_utils::intern_kv_pair(name, radiate_utils::intern!(parts.join(".")))
    }

    fn params(&self) -> Parameter {
        Parameter::typed::<Self>()
    }

    fn select(
        &self,
        population: &Population<C>,
        optimize: &Objective,
        count: usize,
    ) -> Population<C>;
}
