use super::{Alter, AlterAction, Mutate};
use crate::{random_provider, Chromosome, EngineCompoment};

pub struct SwapMutator {
    rate: f32,
}

impl SwapMutator {
    pub fn new(rate: f32) -> Self {
        SwapMutator { rate }
    }
}

impl EngineCompoment for SwapMutator {
    fn name(&self) -> &'static str {
        "SwapMutator"
    }
}

impl<C: Chromosome> Alter<C> for SwapMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome> Mutate<C> for SwapMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;

        for i in 0..chromosome.len() {
            if random_provider::random::<f32>() < self.rate {
                let swap_index = random_provider::gen_range(0..chromosome.len());

                if swap_index == i {
                    continue;
                }

                chromosome.as_mut().swap(i, swap_index);
                mutations += 1;
            }
        }

        mutations
    }
}
