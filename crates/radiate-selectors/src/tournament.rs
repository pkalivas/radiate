use radiate_core::{Chromosome, Objective, Population, Select, random_provider};

#[derive(Debug, Clone)]
pub struct TournamentSelector {
    num: usize,
}

impl TournamentSelector {
    pub fn new(num: usize) -> Self {
        TournamentSelector { num: num.max(1) }
    }
}

impl<C: Chromosome + Clone> Select<C> for TournamentSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let n = population.len();
        if n == 0 || count == 0 {
            return Population::new(Vec::new());
        }

        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let mut best = random_provider::range(0..n);
            for _ in 1..self.num {
                let r = random_provider::range(0..n);
                if r < best {
                    best = r;
                }
            }

            selected.push(population[best].clone());
        }

        Population::new(selected)
    }
}
