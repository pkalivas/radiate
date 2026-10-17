use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Phenotype, Select};

/// Linear ranking (Baker, 1985). The population is assumed to be sorted best-first by
/// the engine, so index 0 is the best. The best individual is selected with
/// probability `pressure / N`, the worst with `(2 - pressure) / N`, and everything
/// in between is interpolated linearly by rank.
///
/// `pressure` must be in `[1.0, 2.0]`: `1.0` is uniform selection, `2.0` gives the
/// worst individual zero chance of being selected.
#[derive(Debug, Clone)]
pub struct LinearRankSelector {
    selection_pressure: f32,
}

impl LinearRankSelector {
    pub fn new(selection_pressure: f32) -> Self {
        assert!(
            (1.0..=2.0).contains(&selection_pressure),
            "LinearRankSelector pressure must be in [1.0, 2.0], got {selection_pressure}"
        );

        LinearRankSelector { selection_pressure }
    }
}

impl<C: Chromosome> Select<C> for LinearRankSelector {
    fn select(&self, population: &[Phenotype<C>], _: &Objective, count: usize) -> Vec<usize> {
        let n = population.len();
        if n == 0 || count == 0 {
            return Vec::new();
        }

        if n == 1 {
            return vec![0; count];
        }

        let n_plus = self.selection_pressure;
        let n_minus = 2.0 - n_plus;
        let step = (n_plus - n_minus) / (n - 1) as f32;

        let weights = (0..n)
            .map(|i| (n_plus - step * i as f32) / n as f32)
            .collect::<Vec<_>>();

        ProbabilityWheelIterator::new(weights, count).collect()
    }
}
