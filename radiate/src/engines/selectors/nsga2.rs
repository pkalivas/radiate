use crate::objectives::{pareto, Objective};
use crate::{Chromosome, Population, Select};

pub struct NSGA2Selector;

impl NSGA2Selector {
    pub fn new() -> Self {
        NSGA2Selector
    }
}

impl<C: Chromosome> Select<C> for NSGA2Selector {
    fn name(&self) -> &'static str {
        "NSGA2 Selector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let scores = population
            .iter()
            .map(|individual| individual.score().as_ref().unwrap().clone())
            .collect::<Vec<_>>();

        let ranks = pareto::rank(population, objective);
        let distances = pareto::crowding_distance(&scores, objective);

        let mut indices: Vec<usize> = (0..population.len()).collect();

        indices.sort_by(|&a, &b| {
            let a_rank = ranks[a];
            let b_rank = ranks[b];
            let a_distance = distances[a];
            let b_distance = distances[b];

            if a_rank < b_rank || (a_rank == b_rank && a_distance > b_distance) {
                std::cmp::Ordering::Less
            } else if b_rank < a_rank || (b_rank == a_rank && b_distance > a_distance) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });

        indices
            .iter()
            .take(count)
            .map(|&i| population.get(i).clone())
            .collect::<Population<C>>()
    }
}
