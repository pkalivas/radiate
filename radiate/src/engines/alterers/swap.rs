use super::{AlterAction, AlterResult, Alterer, IntoAlter, Mutate};
use crate::{Chromosome, random_provider};

pub struct SwapMutator {
    rate: f32,
}

impl SwapMutator {
    pub fn new(rate: f32) -> Self {
        SwapMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for SwapMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        for i in 0..chromosome.len() {
            if random_provider::random::<f32>() < rate {
                let swap_index = random_provider::random_range(0..chromosome.len());

                if swap_index == i {
                    continue;
                }

                chromosome.as_mut().swap(i, swap_index);
                mutations += 1;
            }
        }

        AlterResult(mutations, None)
    }
}

impl<C: Chromosome> IntoAlter<C> for SwapMutator {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "SwapMutator",
            self.rate,
            AlterAction::Mutate(Box::new(self)),
        )
    }
}
