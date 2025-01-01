use super::{AlterAction, Alter, EngineCompoment, Mutate};
use crate::{random_provider, Chromosome};

pub struct BitFlipMutator {
    rate: f32,
}

impl BitFlipMutator {
    pub fn new(rate: f32) -> Self {
        BitFlipMutator { rate }
    }
}

impl EngineCompoment for BitFlipMutator {
    fn name(&self) -> &'static str {
        "BitFlipMutator"
    }
}

impl<C: Chromosome> Alter<C> for BitFlipMutator
where
    C::Gene: Clone + std::ops::BitXor<Output = C::Gene> + From<u8>,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome> Mutate<C> for BitFlipMutator
where
    C::Gene: Clone + std::ops::BitXor<Output = C::Gene> + From<u8>,
{
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;

        for gene in chromosome.iter_mut() {
            if random_provider::random::<f32>() < self.rate {
                let bit_position = random_provider::gen_range(0..8);
                let mask = C::Gene::from(1 << bit_position);
                *gene = gene.clone() ^ mask;
                mutations += 1;
            }
        }

        mutations
    }
}
