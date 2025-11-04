use radiate_core::{AlterResult, Chromosome, Mutate, random_provider};

/// The [SwapMutator] is a simple mutator that swaps random genes in the [Chromosome].
#[derive(Debug, Clone)]
pub struct SwapMutator {
    rate: f32,
}

impl SwapMutator {
    pub fn new(rate: f32) -> Self {
        SwapMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for SwapMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        for i in 0..chromosome.len() {
            if random_provider::bool(rate) {
                let swap_index = random_provider::range(0..chromosome.len());

                if swap_index == i {
                    continue;
                }

                chromosome.genes_mut().swap(i, swap_index);
                mutations += 1;
            }
        }

        AlterResult::from(mutations)
    }
}
