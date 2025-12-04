pub mod botzmann;
pub mod elite;
pub mod linear_rank;
pub mod nsga2;
pub mod random_selector;
pub mod rank;
pub mod roulette;
pub mod steady_state;
pub mod stochastic_sampling;
pub mod tournament;

use radiate_core::random_provider;

pub use botzmann::BoltzmannSelector;
pub use elite::EliteSelector;
pub use linear_rank::LinearRankSelector;
pub use nsga2::{NSGA2Selector, TournamentNSGA2Selector};
pub use random_selector::RandomSelector;
pub use rank::RankSelector;
pub use roulette::RouletteSelector;
pub use steady_state::SteadyStateSelector;
pub use stochastic_sampling::StochasticUniversalSamplingSelector;
pub use tournament::TournamentSelector;

// /// An iterator that generates random indices based on probabilities.
// /// This iterator is used in the RouletteWheel selection algorithm, and
// /// Boltzmann selection algorithm. This is essentially the 'roulette wheel'
// /// that is spun to select individuals from the population. The probability
// /// of selecting an individual is based on the fitness (probability) of the individual.
// /// The higher the fitness, the higher the probability of the individual being selected.
pub(crate) struct ProbabilityWheelIterator {
    cdf: Vec<f32>,
    max_index: usize,
    current: usize,
    uniform: bool,
}

impl ProbabilityWheelIterator {
    pub fn new(probabilities: &[f32], max_index: usize) -> Self {
        let mut cdf = Vec::with_capacity(probabilities.len());
        let mut total = 0.0f32;

        for &p in probabilities {
            let w = if p.is_finite() && p > 0.0 { p } else { 0.0 };
            total += w;
            cdf.push(total);
        }

        let uniform = !total.is_finite() || total <= 0.0;
        if !uniform && total != 1.0 {
            let inv = 1.0 / total;
            for v in &mut cdf {
                *v *= inv;
            }
        }

        Self {
            cdf,
            max_index,
            current: 0,
            uniform,
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

        let n = self.cdf.len();
        if n == 0 {
            self.current += 1;
            return Some(0);
        }

        let idx = if self.uniform {
            let i = (random_provider::random::<f32>() * n as f32) as usize;
            i.min(n.saturating_sub(1))
        } else {
            // threshold in [0, 1)
            let r = random_provider::random::<f32>();
            let i = self
                .cdf
                .binary_search_by(|v| v.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Less))
                .unwrap_or_else(|i| i);
            i.min(n - 1)
        };

        self.current += 1;
        Some(idx)
    }
}
