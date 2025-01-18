use crate::objectives::Objective;
use crate::{random_provider, Chromosome, EngineCompoment, Population, Select};

// In rank selection, the selection probability does not depend directly on the fitness, but on
// the fitness rank of an individual within the population. This puts large fitness differences
// into perspective; moreover, the exact fitness values themselves do not have to be available,
// but only a sorting of the individuals according to quality.
pub struct RankSelector;

impl RankSelector {
    pub fn new() -> Self {
        RankSelector
    }
}

impl EngineCompoment for RankSelector {
    fn name(&self) -> &'static str {
        "RankSelector"
    }
}

impl<C: Chromosome> Select<C> for RankSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        let mut rank_sum = 0.0;
        for i in 0..population.len() {
            rank_sum += (i + 1) as f32;
        }

        let mut probabilities = Vec::with_capacity(population.len());
        for i in 0..population.len() {
            probabilities.push((population.len() as f32 - i as f32) / rank_sum);
        }

        for _ in 0..count {
            let mut r = random_provider::random::<f32>();
            let mut i = 0;
            while r > probabilities[i] {
                r -= probabilities[i];
                i += 1;
            }
            selected.push(population[i].clone());
        }

        selected.into_iter().collect()
    }
}
