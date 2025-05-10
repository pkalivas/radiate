use radiate_core::{
    Chromosome, Objective, Optimize, Population, ProbabilityWheelIterator, Select, pareto,
};

#[derive(Debug, Default)]
pub struct RouletteSelector;

impl RouletteSelector {
    pub fn new() -> Self {
        RouletteSelector
    }
}

impl<C: Chromosome> Select<C> for RouletteSelector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let scores = population
                    .get_scores()
                    .iter()
                    .map(|score| score.as_f32())
                    .collect::<Vec<f32>>();

                let total = scores.iter().sum::<f32>();
                let mut fitness_values =
                    scores.iter().map(|&fit| fit / total).collect::<Vec<f32>>();

                if let Optimize::Minimize = opt {
                    fitness_values.reverse();
                }

                fitness_values
            }
            Objective::Multi(_) => {
                let weights = pareto::weights(&population.get_scores(), objective);
                let total_weights = weights.iter().sum::<f32>();
                weights
                    .iter()
                    .map(|&fit| fit / total_weights)
                    .collect::<Vec<f32>>()
            }
        };

        ProbabilityWheelIterator::new(&fitness_values, count)
            .map(|idx| population[idx].clone())
            .collect::<Population<C>>()
    }
}
