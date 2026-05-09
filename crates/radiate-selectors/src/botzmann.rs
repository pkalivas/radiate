use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Optimize, Population, Select, math::norm, pareto};
use radiate_utils::MinMax;

const MIN: f32 = 1e-6;

#[derive(Debug, Clone, Default)]
pub struct BoltzmannSelector {
    temperature: f32,
}

impl BoltzmannSelector {
    pub fn new(temperature: f32) -> Self {
        BoltzmannSelector { temperature }
    }

    fn apply_boltzmann(&self, weights: &mut [f32]) {
        let mut minmax = MinMax::default();
        for &score in weights.iter() {
            minmax.add(&score);
        }

        let min = minmax.min();
        let diff = minmax.range().abs().max(MIN);

        for score in weights.iter_mut() {
            *score = (self.temperature * ((*score - min) / diff)).exp();
        }
    }
}

impl<C: Chromosome + Clone> Select<C> for BoltzmannSelector {
    #[inline]
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut fitness_values = population
                    .iter_scores()
                    .filter_map(|s| s.first())
                    .collect::<Vec<_>>();

                self.apply_boltzmann(&mut fitness_values);
                norm::scale_l1_affine_sorted(&mut fitness_values);

                if let Optimize::Minimize = opt {
                    fitness_values.reverse();
                }

                fitness_values
            }
            Objective::Multi(_) => {
                let scores = population.iter_scores().collect::<Vec<_>>();

                let mut weights = pareto::weights(&scores, objective);
                self.apply_boltzmann(&mut weights);
                norm::scale_l1(&mut weights);

                weights
            }
        };

        ProbabilityWheelIterator::new(&fitness_values, count)
            .map(|idx| population[idx].clone())
            .collect::<Population<C>>()
    }

    #[inline]
    fn select_idx(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Vec<usize> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut fitness_values = population
                    .iter_scores()
                    .filter_map(|s| s.first())
                    .collect::<Vec<_>>();

                self.apply_boltzmann(&mut fitness_values);
                norm::scale_l1_affine_sorted(&mut fitness_values);

                if let Optimize::Minimize = opt {
                    fitness_values.reverse();
                }

                fitness_values
            }
            Objective::Multi(_) => {
                let scores = population.iter_scores().collect::<Vec<_>>();

                let mut weights = pareto::weights(&scores, objective);
                self.apply_boltzmann(&mut weights);
                norm::scale_l1(&mut weights);

                weights
            }
        };

        ProbabilityWheelIterator::new(&fitness_values, count).collect()
    }
}
