pub mod botzmann;
pub mod elite;
pub mod linear_rank;
pub mod monte_carlo;
pub mod nsga2;
pub mod rank;
pub mod roulette;
pub mod steady_state;
pub mod stochastic_sampling;
pub mod tournament;

use crate::engines::genome::population::Population;
use crate::objectives::Objective;
use crate::Chromosome;
pub use botzmann::*;
pub use elite::*;
pub use linear_rank::*;
pub use monte_carlo::*;
pub use nsga2::*;
pub use rank::*;
pub use roulette::*;
pub use steady_state::*;
pub use stochastic_sampling::*;
pub use tournament::*;

pub trait Select<C: Chromosome> {
    fn name(&self) -> &'static str;

    fn select(
        &self,
        population: &Population<C>,
        optimize: &Objective,
        count: usize,
    ) -> Population<C>;
}

pub(super) struct ProbabilityIterator<'a> {
    probabilities: &'a [f32],
    total: f32,
    max_index: usize,
    current: usize,
}

impl<'a> ProbabilityIterator<'a> {
    pub fn new(probabilities: &'a [f32], max_index: usize) -> Self {
        let total = probabilities.iter().sum();
        Self {
            probabilities,
            total,
            max_index,
            current: 0,
        }
    }
}

impl<'a> Iterator for ProbabilityIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.max_index {
            return None;
        }

        let mut value = rand::random::<f32>() * self.total;
        let mut index = 0;
        
        for (i, &prob) in self.probabilities.iter().enumerate() {
            value -= prob;
            if value <= 0.0 {
                index = i;
                break;
            }
        }

        self.current += 1;
        Some(index)
    }
}