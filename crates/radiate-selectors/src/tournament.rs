use radiate_core::{Chromosome, Objective, Phenotype, Select, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl<C: Chromosome> Select<C> for TournamentSelector {
    fn select(&self, population: &[Phenotype<C>], _: &Objective, count: usize) -> Vec<usize> {
        let n = population.len();
        if n == 0 || count == 0 {
            return Vec::new();
        }

        random_provider::with_rng(|rand| {
            let mut selected = Vec::with_capacity(count);

            for _ in 0..count {
                let mut best = rand.range(0..n);
                for _ in 1..self.k {
                    let r = rand.range(0..n);
                    if r < best {
                        best = r;
                    }
                }

                selected.push(best);
            }

            selected
        })
    }
}
