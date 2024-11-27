use super::Select;
use crate::objectives::Objective;
use crate::{random_provider, Chromosome, Population};

pub struct TournamentSelector {
    num: usize,
}

impl TournamentSelector {
    pub fn new(num: usize) -> Self {
        Self { num }
    }
}

impl<C: Chromosome> Select<C> for TournamentSelector {
    fn name(&self) -> &'static str {
        "Tournament Selector"
    }

    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let mut tournament = Vec::with_capacity(self.num);
            for _ in 0..self.num {
                let idx = random_provider::gen_range(0..population.len());
                tournament.push(idx);
            }

            tournament.sort();

            selected.push(population.get(tournament[0]).clone());
        }

        Population::from_vec(selected)
    }
}
