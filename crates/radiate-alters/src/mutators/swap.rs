use radiate_core::{AlterResult, Chromosome, Mutate, random_provider};

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
            if random_provider::random::<f32>() < rate {
                let swap_index = random_provider::range(0..chromosome.len());

                if swap_index == i {
                    continue;
                }

                chromosome.as_mut().swap(i, swap_index);
                mutations += 1;
            }
        }

        mutations.into()
    }
}
