pub mod botzmann;
pub mod elite;
pub mod linear_rank;
pub mod nsga2;
pub mod nsga3;
pub mod random_selector;
pub mod rank;
pub mod roulette;
pub mod stochastic_sampling;
pub mod tournament;

use radiate_core::random_provider;

pub use botzmann::BoltzmannSelector;
pub use elite::EliteSelector;
pub use linear_rank::LinearRankSelector;
pub use nsga2::{NSGA2Selector, TournamentNSGA2Selector};
pub use nsga3::NSGA3Selector;
pub use random_selector::RandomSelector;
pub use rank::RankSelector;
pub use roulette::RouletteSelector;
pub use stochastic_sampling::StochasticUniversalSamplingSelector;
pub use tournament::TournamentSelector;

pub(crate) struct ProbabilityWheelIterator<'a> {
    probs: &'a [f32],
    total: f32,
    max_index: usize,
    current: usize,
}

impl<'a> ProbabilityWheelIterator<'a> {
    pub fn new(weights: &'a [f32], max_index: usize) -> Self {
        let total = weights.iter().sum::<f32>();

        Self {
            probs: weights,
            total,
            max_index,
            current: 0,
        }
    }
}

impl<'a> Iterator for ProbabilityWheelIterator<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.max_index {
            return None;
        }

        self.current += 1;

        let n = self.probs.len();
        if n == 0 {
            return Some(0);
        }

        let mark = random_provider::range(0_f32..self.total);

        let mut accum = 0.0;
        for (i, &p) in self.probs.iter().enumerate() {
            accum += p;
            if accum >= mark {
                return Some(i);
            }
        }

        Some(n - 1)
    }
}
