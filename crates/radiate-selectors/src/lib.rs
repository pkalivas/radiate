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

use radiate_core::{Optimize, random_provider};

pub use botzmann::BoltzmannSelector;
pub use elite::EliteSelector;
pub use linear_rank::LinearRankSelector;
pub use nsga2::{NSGA2Selector, TournamentNSGA2Selector};
pub use nsga3::NSGA3Selector;
use radiate_utils::MinMax;
pub use random_selector::RandomSelector;
pub use rank::RankSelector;
pub use roulette::RouletteSelector;
pub use stochastic_sampling::StochasticUniversalSamplingSelector;
pub use tournament::TournamentSelector;

/// Map raw single-objective scores to non-negative weights where larger is better,
/// in place and independent of slice order. Maximizing keeps today's behavior
/// (raw score, shifted only if negatives exist); minimizing uses `max - s`. `EPS`
/// keeps the worst individual at a small nonzero chance.
pub(crate) fn directed_weights(scores: &mut [f32], opt: Optimize) {
    let (min, max) = scores.iter().collect::<MinMax<f32>>().min_max();
    let offset = if min < 0.0 { -min } else { 0.0 };

    for s in scores.iter_mut() {
        *s = match opt {
            Optimize::Maximize => *s + offset,
            Optimize::Minimize => max - *s,
        } + f32::EPSILON;
    }
}

pub(crate) struct ProbabilityWheelIterator {
    cdf: Vec<f32>,
    total: f32,
    max_index: usize,
    current: usize,
    n: usize,
}

impl ProbabilityWheelIterator {
    pub fn new(mut weights: Vec<f32>, max_index: usize) -> Self {
        let mut running = 0.0;
        let n = weights.len();
        for w in weights.iter_mut() {
            running += *w;
            *w = running;
        }

        let total = running;
        Self {
            cdf: weights,
            total,
            max_index,
            current: 0,
            n,
        }
    }
}

impl Iterator for ProbabilityWheelIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.current >= self.max_index {
            return None;
        }

        self.current += 1;

        if self.n == 0 {
            return Some(0);
        }

        let mark = random_provider::range(0_f32..self.total);
        let idx = self.cdf.partition_point(|&c| c < mark);

        Some(idx.min(self.n - 1))
    }
}
