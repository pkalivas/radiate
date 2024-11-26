pub mod botzmann;
pub mod elite;
pub mod rank;
pub mod roulette;
pub mod tournament;

pub use botzmann::*;
pub use elite::*;
pub use rank::*;
pub use roulette::*;
pub use tournament::*;

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
