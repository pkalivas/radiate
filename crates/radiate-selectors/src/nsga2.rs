use radiate_core::{Chromosome, Objective, Population, Select, pareto};

/// NSGA2 Selector. Selects individuals based on the NSGA2 algorithm.
/// This algorithm ranks individuals based on their dominance relationships
/// with other individuals in the population. The result is a vector of ranks,
/// where the rank of the individual at index `i` is `ranks[i]`.
/// Individuals are then selected based on their rank and crowding distance.
/// The crowding distance is a measure of how close an individual is to its
/// neighbors in the objective space. Individuals with a higher crowding distance
/// are more desirable because they are more spread out. This is useful for selecting
/// diverse solutions in a multi-objective optimization problem. It uses 'fast non-dominated sorting'
pub struct NSGA2Selector;

impl NSGA2Selector {
    pub fn new() -> Self {
        NSGA2Selector
    }
}

impl<C: Chromosome> Select<C> for NSGA2Selector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let scores = population.get_scores();
        let ranks = pareto::rank(scores.as_slice(), objective);
        let distances = pareto::crowding_distance(&scores, objective);

        let mut indices = (0..population.len()).collect::<Vec<usize>>();

        // This is commonly called "non-dominated sorting" in the NSGA-II algorithm.
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
            .map(|&i| population[i].clone())
            .collect::<Population<C>>()
    }
}
