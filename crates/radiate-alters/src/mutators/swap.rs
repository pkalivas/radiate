use radiate_core::{AlterResult, Chromosome, Mutate, Rate, random_provider};

/// The [SwapMutator] is a simple mutator that swaps random genes in the [Chromosome].
#[derive(Debug, Clone)]
pub struct SwapMutator {
    rate: Rate,
}

impl SwapMutator {
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        SwapMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for SwapMutator {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        random_provider::with_rng(|rand| {
            for i in 0..chromosome.len() {
                if rand.bool(rate) {
                    let swap_index = rand.range(0..chromosome.len());
                    if swap_index == i {
                        continue;
                    }

                    chromosome.genes_mut().swap(i, swap_index);
                    mutations += 1;
                }
            }
        });

        AlterResult::from(mutations)
    }
}
