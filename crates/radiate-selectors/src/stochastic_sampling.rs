use radiate_core::{
    Chromosome, Objective, Optimize, Phenotype, Select, math::norm, pareto, random_provider,
};

#[derive(Debug, Clone, Default)]
pub struct StochasticUniversalSamplingSelector;

impl StochasticUniversalSamplingSelector {
    pub fn new() -> Self {
        StochasticUniversalSamplingSelector
    }
}

impl<C: Chromosome + Clone> Select<C> for StochasticUniversalSamplingSelector {
    fn select(
        &self,
        population: &[Phenotype<C>],
        objective: &Objective,
        count: usize,
    ) -> Vec<usize> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut weights = population
                    .iter()
                    .filter_map(|p| p.score())
                    .filter_map(|score| score.first())
                    .collect::<Vec<f32>>();

                norm::scale_l1(&mut weights);

                if let Optimize::Minimize = opt {
                    weights.reverse();
                }

                weights
            }
            Objective::Multi(_) => {
                let scores = population
                    .iter()
                    .filter_map(|p| p.score())
                    .collect::<Vec<_>>();
                let mut weights = pareto::weights(&scores, objective);

                norm::scale_l1(&mut weights);

                weights
            }
        };

        let fitness_total = fitness_values.iter().sum::<f32>();
        let point_distance = fitness_total / count as f32;
        let start_point = random_provider::range(0.0..point_distance);

        let mut pointers = Vec::with_capacity(count);
        let mut current_point = start_point;

        for _ in 0..count {
            let mut index = 0;
            let mut fitness_sum = fitness_values[index];
            while fitness_sum < current_point && index < fitness_values.len() - 1 {
                index += 1;
                fitness_sum += fitness_values[index];
            }
            pointers.push(index);
            current_point += point_distance;
        }

        pointers
    }
}
