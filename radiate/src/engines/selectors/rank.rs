use crate::{Chromosome, Optimize, Population, RandomProvider, Select};

pub struct RankSelector;

impl RankSelector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RankSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Chromosome> Select<C> for RankSelector {
    fn name(&self) -> &'static str {
        "Rank Selector"
    }

    fn select(&self, population: &Population<C>, _: &Optimize, count: usize) -> Population<C> {
        // TODO: This is wrong, fix me.
        let mut selected = Vec::with_capacity(count);

        let total_rank = (population.len() * (population.len() + 1)) as f32 / 2.0;

        for _ in 0..count {
            let mut idx = RandomProvider::gen_range(0.0..total_rank);
            let mut selected_idx = 0;
            for individual in population.iter() {
                idx -= (population.len() - selected_idx) as f32;
                if idx <= 0.0 {
                    selected.push(individual.clone());
                    break;
                }
                selected_idx += 1;
            }
        }

        Population::from_vec(selected)
    }
}
