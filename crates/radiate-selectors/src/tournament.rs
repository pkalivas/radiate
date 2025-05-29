use radiate_core::{Chromosome, Objective, Population, Select, random_provider};

#[derive(Debug, Clone)]
pub struct TournamentSelector {
    num: usize,
}

impl TournamentSelector {
    pub fn new(num: usize) -> Self {
        TournamentSelector { num }
    }
}

impl<C: Chromosome + Clone> Select<C> for TournamentSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let mut tournament = (0..self.num)
                .map(|_| random_provider::range(0..population.len()))
                .collect::<Vec<usize>>();

            tournament.sort();

            selected.push(population[tournament[0]].clone());
        }

        Population::new(selected)
    }
}
