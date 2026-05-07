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

pub(crate) struct ProbabilityWheelIterator {
    cdf: Vec<f32>,
    total: f32,
    max_index: usize,
    current: usize,
}

impl ProbabilityWheelIterator {
    pub fn new(weights: &[f32], max_index: usize) -> Self {
        let mut cdf = Vec::with_capacity(weights.len());
        let mut total = 0.0f32;

        for &w in weights {
            let w = if w.is_finite() && w > 0.0 { w } else { 0.0 };
            total += w;
            cdf.push(total);
        }

        Self {
            cdf,
            total,
            max_index,
            current: 0,
        }
    }
}

impl Iterator for ProbabilityWheelIterator {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.max_index {
            return None;
        }
        self.current += 1;

        let n = self.cdf.len();
        if n == 0 {
            return Some(0);
        }

        let idx = if self.total > 0.0 && self.total.is_finite() {
            let r = random_provider::random::<f32>() * self.total;
            self.cdf.partition_point(|&v| v <= r).min(n - 1)
        } else {
            let i = (random_provider::random::<f32>() * n as f32) as usize;
            i.min(n - 1)
        };

        Some(idx)
    }
}
