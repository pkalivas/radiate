use radiate_core::{Chromosome, Objective, Population, Select, random_provider};

pub struct TournamentSelector {
    num: usize,
}

impl TournamentSelector {
    pub fn new(num: usize) -> Self {
        TournamentSelector { num }
    }
}

impl<C: Chromosome> Select<C> for TournamentSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let mut tournament = Vec::with_capacity(self.num);
            for _ in 0..self.num {
                let idx = random_provider::range(0..population.len());
                tournament.push(idx);
            }

            tournament.sort();

            selected.push(population[tournament[0]].clone());
        }

        Population::new(selected)
    }
}
