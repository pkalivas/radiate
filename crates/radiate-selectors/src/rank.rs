use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Population, Select};

// In rank selection, the selection probability does not depend directly on the fitness, but on
// the fitness rank of an individual within the population. This puts large fitness differences
// into perspective; moreover, the exact fitness values themselves do not have to be available,
// but only a sorting of the individuals according to quality.
#[derive(Debug, Default)]
pub struct RankSelector;

impl RankSelector {
    pub fn new() -> Self {
        RankSelector
    }
}

impl<C: Chromosome + Clone> Select<C> for RankSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let n = population.len();
        if n == 0 || count == 0 {
            return Population::new(Vec::new());
        }

        let rank_sum = (1..=n).map(|i| i as f32).sum::<f32>();
        let mut probabilities = Vec::with_capacity(n);
        for i in 0..n {
            probabilities.push((n as f32 - i as f32) / rank_sum);
        }

        ProbabilityWheelIterator::new(&probabilities, count)
            .map(|i| population[i].clone())
            .collect()
    }
}
