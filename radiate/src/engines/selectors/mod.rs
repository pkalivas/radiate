pub mod botzmann;
pub mod elite;
pub mod linear_rank;
pub mod rank;
pub mod roulette;
pub mod steady_state;
pub mod stochastic_sampling;
pub mod tournament;
pub mod nsga2;

pub use botzmann::*;
pub use elite::*;
pub use linear_rank::*;
pub use rank::*;
pub use roulette::*;
pub use steady_state::*;
pub use stochastic_sampling::*;
pub use tournament::*;
pub use nsga2::*;

use crate::engines::genome::population::Population;
use crate::{Chromosome, Objective, Optimize};

pub trait Select<C: Chromosome> {
    fn name(&self) -> &'static str;

    fn select(
        &self,
        population: &Population<C>,
        optimize: &Objective,
        count: usize,
    ) -> Population<C>;
}
