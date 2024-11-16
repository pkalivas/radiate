use rand::Rng;

use crate::{Gene, Optimize, Population};

use super::Select;

pub struct Tournament {
    num: usize,
}

impl Tournament {
    pub fn new(num: usize) -> Self {
        Self { num }
    }
}

impl<G: Gene<G, A>, A> Select<G, A> for Tournament {
    fn select(
        &self,
        population: &Population<G, A>,
        _: &Optimize,
        count: usize,
    ) -> Population<G, A> {
        let mut rng = rand::thread_rng();
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let mut tournament = Vec::with_capacity(self.num);
            for _ in 0..self.num {
                let idx = rng.gen_range(0..population.len());
                tournament.push(idx);
            }

            tournament.sort();

            selected.push(population.get(tournament[0]).clone());
        }

        Population::from_vec(selected)
    }
}
