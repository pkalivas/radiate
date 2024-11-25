use crate::engines::genome::population::Population;
use crate::{Chromosome, Optimize};

pub trait Select<C: Chromosome> {
    fn name(&self) -> &'static str;

    fn select(
        &self,
        population: &Population<C>,
        optimize: &Optimize,
        count: usize,
    ) -> Population<C>;
}
