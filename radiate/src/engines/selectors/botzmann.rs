use super::Select;
use crate::objectives::{Objective, Optimize};
use crate::{random_provider, Chromosome, Population};

pub struct BoltzmannSelector {
    temperature: f32,
}

impl BoltzmannSelector {
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }
}

impl<C: Chromosome> Select<C> for BoltzmannSelector {
    fn name(&self) -> &'static str {
        "Boltzmann Selector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        let mut min = population.get(0).score().as_ref().unwrap().as_float();
        let mut max = min;

        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_float();
            if score < min {
                min = score;
            }
            if score > max {
                max = score;
            }
        }

        let diff = max - min;
        if diff == 0.0 {
            return population
                .iter()
                .take(count)
                .cloned()
                .collect::<Population<C>>();
        }

        let mut result = Vec::with_capacity(population.len());
        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_float();
            let fitness = (score - min) / diff;
            let value = (self.temperature * fitness).exp();

            result.push(value);
        }

        let total_fitness = result.iter().sum::<f32>();
        for i in 0..result.len() {
            result[i] /= total_fitness;
        }

        match objective {
            Objective::Single(opt) => {
                if opt == &Optimize::Minimize {
                    result.reverse();
                }
            }
            Objective::Multi(_) => {}
        }

        let total_fitness = result.iter().sum::<f32>();

        for _ in 0..count {
            let mut idx = random_provider::gen_range(0.0..total_fitness);

            for (i, val) in result.iter().enumerate() {
                idx -= val;
                if idx <= 0.0 {
                    selected.push(population.get(i).clone());
                    break;
                }
            }
        }

        Population::from_vec(selected)
    }
}

// fn select(
//     &self,
//     population: &Population<C>,
//     _optimize: &Optimize,
//     count: usize,
// ) -> Population<C> {
//     let mut boltzmann_fitnesses: Vec<f32> = population
//         .iter()
//         .map(|ind| (ind.fitness() / self.temperature).exp())
//         .collect();
//     let total_boltzmann_fitness: f32 = boltzmann_fitnesses.iter().sum();
//
//     let mut selected_population = Population::new();
//     for _ in 0..count {
//         let spin = random_provider::gen_range(0.0..total_boltzmann_fitness);
//         let mut cumulative_fitness = 0.0;
//
//         for (i, individual) in population.iter().enumerate() {
//             cumulative_fitness += boltzmann_fitnesses[i];
//             if cumulative_fitness >= spin {
//                 selected_population.add(individual.clone());
//                 break;
//             }
//         }
//     }
//
//     selected_population
// }
