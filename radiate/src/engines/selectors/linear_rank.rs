use crate::objectives::{Objective, Optimize};
use crate::{random_provider, Chromosome, Population, Select};

pub struct LinearRankSelector {
    selection_pressure: f32,
}

impl LinearRankSelector {
    pub fn new(selection_pressure: f32) -> Self {
        LinearRankSelector { selection_pressure }
    }
}

impl<C: Chromosome> Select<C> for LinearRankSelector {
    fn name(&self) -> &'static str {
        "LinearRankSelector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let mut fitness_values = population
            .iter()
            .map(|individual| individual.score().as_ref().unwrap().as_f32())
            .collect::<Vec<f32>>();

        let total_rank: f32 = (1..=fitness_values.len()).map(|i| i as f32).sum();
        let mut selected_population = Vec::with_capacity(count);

        match objective {
            Objective::Single(opt) => {
                if opt == &Optimize::Minimize {
                    fitness_values.reverse();
                }
            }
            Objective::Multi(_) => {}
        }

        for fit in fitness_values.iter_mut() {
            *fit = 1.0 / *fit;
        }

        for _ in 0..count {
            let target = random_provider::gen_range(0.0..total_rank);
            let mut cumulative_rank = 0.0;

            for (rank, _) in fitness_values.iter().enumerate() {
                cumulative_rank += (rank + 1) as f32 * self.selection_pressure;
                if cumulative_rank > target {
                    selected_population.push(population[rank].clone());
                    break;
                }
            }
        }

        Population::from_vec(selected_population)
    }
}
