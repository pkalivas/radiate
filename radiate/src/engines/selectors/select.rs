use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::population::Population;
use crate::Optimize;

pub trait Select<G, A>
where
    G: Gene<G, A>,
{
    fn select(
        &self,
        population: &Population<G, A>,
        optimize: &Optimize,
        count: usize,
    ) -> Population<G, A>;
}
