use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome};

pub struct InversionMutator {
    rate: f32,
}

impl InversionMutator {
    pub fn new(rate: f32) -> Self {
        InversionMutator { rate }
    }
}

impl<C: Chromosome> Alter<C> for InversionMutator {
    fn name(&self) -> &'static str {
        "InversionMutator"
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

        if random_provider::random::<i32>() < range {
            let start = random_provider::gen_range(0..chromosome.len());
            let end = random_provider::gen_range(start..chromosome.len());

            chromosome.get_genes_mut()[start..end].reverse();
            mutations += 1;
        }

        mutations
    }
}