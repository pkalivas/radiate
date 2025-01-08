use crate::{random_provider, Chromosome};

use super::{Alter, AlterAction, EngineCompoment, Mutate};

pub struct InversionMutator {
    rate: f32,
}

impl InversionMutator {
    pub fn new(rate: f32) -> Self {
        InversionMutator { rate }
    }
}

impl EngineCompoment for InversionMutator {
    fn name(&self) -> &'static str {
        "InversionMutator"
    }
}

impl<C: Chromosome> Alter<C> for InversionMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome> Mutate<C> for InversionMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;

        if random_provider::random::<f32>() < self.rate {
            let start = random_provider::gen_range(0..chromosome.len());
            let end = random_provider::gen_range(start..chromosome.len());

            chromosome.as_mut()[start..end].reverse();
            mutations += 1;
        }

        mutations
    }
}
