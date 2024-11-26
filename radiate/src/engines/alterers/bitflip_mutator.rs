use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome};

pub struct BitFlipMutator {
    rate: f32,
}

impl BitFlipMutator {
    pub fn new(rate: f32) -> Self {
        BitFlipMutator { rate }
    }
}

impl<C: Chromosome> Alter<C> for BitFlipMutator
where
    C::GeneType: Clone + std::ops::BitXor<Output = C::GeneType> + From<u8>,
{
    fn name(&self) -> &'static str {
        "BitFlipMutator"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Mutator
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, range: i32) -> i32 {
        let mut mutations = 0;

        for gene in chromosome.iter_mut() {
            if random_provider::random::<i32>() < range {
                let bit_position = random_provider::gen_range(0..8); // Assuming 8 bits per gene
                let mask = C::GeneType::from(1 << bit_position);
                *gene = gene.clone() ^ mask;
                mutations += 1;
            }
        }

        mutations
    }
}