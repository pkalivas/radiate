use radiate_core::{Chromosome, Objective, Population, Select, indexes, pareto, random_provider};

/// NSGA2 Selector. Selects individuals based on the NSGA2 algorithm.
/// This algorithm ranks individuals based on their dominance relationships
/// with other individuals in the population. The result is a vector of ranks,
/// where the rank of the individual at index `i` is `ranks[i]`.
/// Individuals are then selected based on their rank and crowding distance.
/// The crowding distance is a measure of how close an individual is to its
/// neighbors in the objective space. Individuals with a higher crowding distance
/// are more desirable because they are more spread out. This is useful for selecting
/// diverse solutions in a multi-objective optimization problem. It uses 'fast non-dominated sorting'
#[derive(Debug, Clone)]
pub struct NSGA2Selector;

impl NSGA2Selector {
    pub fn new() -> Self {
        NSGA2Selector
    }
}

impl<C: Chromosome + Clone> Select<C> for NSGA2Selector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let scores = population.get_scores().collect::<Vec<_>>();
        let ranks = pareto::rank(&scores, objective);
        let distances = pareto::crowding_distance(&scores);

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

#[derive(Debug, Clone)]
pub struct TournamentNSGA2Selector;

impl TournamentNSGA2Selector {
    pub fn new() -> Self {
        TournamentNSGA2Selector
    }
}

impl<C: Chromosome + Clone> Select<C> for TournamentNSGA2Selector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let scores = population.get_scores().collect::<Vec<_>>();
        let ranks = pareto::rank(&scores, objective);
        let distances = pareto::crowding_distance(&scores);

        let mut indices = (0..population.len()).collect::<Vec<usize>>();

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

        let mut result = Vec::new();

        while result.len() < count {
            let k = std::cmp::min(2 * count - result.len(), population.len());
            let g = indexes::subset(population.len(), k, indexes::SubsetMode::StratifiedCorrect);

            for i in 0..g.len() {
                if result.len() >= count {
                    break;
                }

                let one = g[i];
                let two = g[i + 1];

                let desired_index = if indices[one] < indices[two] {
                    one
                } else if indices[one] > indices[two] {
                    two
                } else {
                    *random_provider::choose(&[one, two])
                };

                let pop_list = scores
                    .iter()
                    .enumerate()
                    .filter(|pair| pair.1 == &scores[desired_index])
                    .map(|pair| pair.0)
                    .collect::<Vec<_>>();

                let random_phenotype = *random_provider::choose(&pop_list);
                result.push(population[random_phenotype].clone());
            }
        }

        result.into_iter().take(count).collect::<Population<C>>()
    }
}
