use crate::{pareto, Chromosome, Objective, Population, Score, Select};

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
        let fitness_index_lookup = population
            .iter()
            .enumerate()
            .map(|(i, individual)| (individual.score().as_ref().unwrap().clone(), i))
            .collect::<Vec<(Score, usize)>>();

        let mut scores = fitness_index_lookup
            .iter()
            .map(|(score, _)| score.clone())
            .collect::<Vec<Score>>();

        pareto::crowding_distance_sort(&mut scores, objective);

        let mut selected_population = Vec::with_capacity(count);
        let mut i = 0;
        while selected_population.len() < count {
            selected_population.push(population.get(fitness_index_lookup[i].1).clone());
            i += 1;
        }

        Population::from_vec(selected_population)
    }
}
