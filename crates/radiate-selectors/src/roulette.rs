use radiate_core::{Chromosome, Objective, Optimize, Population, Select, pareto, random_provider};

#[derive(Debug, Default)]
pub struct RouletteSelector;

impl RouletteSelector {
    pub fn new() -> Self {
        RouletteSelector
    }
}

impl<C: Chromosome + Clone> Select<C> for RouletteSelector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut population_scores = Vec::with_capacity(population.len());
                let mut sum = 0.0;
                for score in population.get_scores() {
                    let single_score = score.as_f32();
                    population_scores.push(single_score);
                    sum += single_score;
                }

                for fit in &mut population_scores {
                    *fit /= sum;
                }

                if let Optimize::Minimize = opt {
                    population_scores.reverse();
                }

                population_scores
            }
            Objective::Multi(_) => {
                let weights =
                    pareto::weights(&population.get_scores().collect::<Vec<_>>(), objective);
                let total_weights = weights.iter().sum::<f32>();
                weights
                    .iter()
                    .map(|&fit| fit / total_weights)
                    .collect::<Vec<f32>>()
            }
        };

        let mut cdf = Vec::with_capacity(fitness_values.len());
        let mut acc = 0.0;
        for &p in &fitness_values {
            acc += p;
            cdf.push(acc);
        }
        let total = *cdf.last().unwrap_or(&1.0);

        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let r = random_provider::random::<f32>() * total;
            let idx = cdf
                .binary_search_by(|x| x.partial_cmp(&r).unwrap())
                .unwrap_or_else(|i| i);
            out.push(population[idx].clone());
        }

        out.into()
    }
}
