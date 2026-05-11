use radiate_core::{Chromosome, Objective, Phenotype, Select, random_provider};

#[derive(Debug, Clone)]
pub struct LinearRankSelector {
    selection_pressure: f32,
}

impl LinearRankSelector {
    pub fn new(selection_pressure: f32) -> Self {
        LinearRankSelector { selection_pressure }
    }
}

impl<C: Chromosome + Clone> Select<C> for LinearRankSelector {
    fn select(&self, population: &[Phenotype<C>], _: &Objective, count: usize) -> Vec<usize> {
        let n = population.len();
        if n == 0 || count == 0 {
            return Vec::new();
        }

        // Population is pre-sorted best-first by the engine, so index 0 = best.
        // Assign weight (n - i) to index i so the best individual gets weight n
        // and the worst gets weight 1. Scale by selection_pressure so that
        // total_rank == max cumulative rank and the inner loop always terminates.
        let total_rank = (1..=n).map(|i| i as f32).sum::<f32>() * self.selection_pressure;
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let target = random_provider::range(0.0..total_rank);
            let mut cumulative = 0.0;

            for i in 0..n {
                cumulative += (n - i) as f32 * self.selection_pressure;
                if cumulative > target {
                    selected.push(i);
                    break;
                }
            }
        }

        selected
    }
}
