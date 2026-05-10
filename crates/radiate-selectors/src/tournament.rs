use radiate_core::{Chromosome, Objective, Phenotype, Select, random_provider};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct TournamentSelector {
    k: usize,
}

impl TournamentSelector {
    pub fn new(k: usize) -> Self {
        TournamentSelector { k: k.max(1) }
    }

    pub fn k(&self) -> usize {
        self.k
    }
}

impl<C: Chromosome + Clone> Select<C> for TournamentSelector {
    fn select(&self, population: &[Phenotype<C>], _: &Objective, count: usize) -> Vec<usize> {
        let n = population.len();
        if n == 0 || count == 0 {
            return Vec::new();
        }

        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let mut best = random_provider::range(0..n);
            for _ in 1..self.k {
                let r = random_provider::range(0..n);
                if r < best {
                    best = r;
                }
            }

            selected.push(best);
        }

        selected
    }
}
