use crate::{Gene, Optimize, Population, RandomRegistry};

use super::Select;

pub struct RankSelector;

impl RankSelector {
    pub fn new() -> Self {
        Self
    }
}

impl<G: Gene<G, A>, A> Select<G, A> for RankSelector {
    fn name(&self) -> &'static str {
        "Rank Selector"
    }

    fn select(
        &self,
        population: &Population<G, A>,
        _: &Optimize,
        count: usize,
    ) -> Population<G, A> {
        let mut selected = Vec::with_capacity(count);

        let total_rank = (population.len() * (population.len() + 1)) as f32 / 2.0;

        for _ in 0..count {
            let mut idx = RandomRegistry::gen_range(0.0..total_rank);
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
