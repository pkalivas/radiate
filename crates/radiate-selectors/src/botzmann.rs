use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Optimize, Phenotype, Select, math::norm, pareto};
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

    fn apply_boltzmann(&self, weights: &mut [f32], opt: Optimize) {
        let (min, max) = weights.iter().collect::<MinMax<f32>>().min_max();
        let diff = (max - min).abs().max(MIN);

        for w in weights.iter_mut() {
            let norm = match opt {
                Optimize::Maximize => (*w - min) / diff,
                Optimize::Minimize => (max - *w) / diff,
            };
            *w = (self.temperature * norm).exp();
        }
    }
}

impl<C: Chromosome> Select<C> for BoltzmannSelector {
    #[inline]
    fn select(
        &self,
        population: &[Phenotype<C>],
        objective: &Objective,
        count: usize,
    ) -> Vec<usize> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut fitness_values = population
                    .iter()
                    .filter_map(|p| p.score().and_then(|score| score.first()))
                    .collect::<Vec<_>>();

                self.apply_boltzmann(&mut fitness_values, *opt);

                fitness_values
            }
            Objective::Multi(_) => {
                let scores = population
                    .iter()
                    .filter_map(|p| p.score())
                    .collect::<Vec<_>>();

                let mut weights = pareto::weights(&scores, objective);
                self.apply_boltzmann(&mut weights, Optimize::Maximize);
                norm::scale_l1(&mut weights);

                weights
            }
        };

        ProbabilityWheelIterator::new(fitness_values, count).collect()
    }
}
