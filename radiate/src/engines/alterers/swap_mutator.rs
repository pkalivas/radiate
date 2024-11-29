use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome};

pub struct SwapMutator {
    rate: f32,
}

impl SwapMutator {
    pub fn new(rate: f32) -> Self {
        SwapMutator { rate }
    }
}

impl<C: Chromosome> Alter<C> for SwapMutator {
    fn name(&self) -> &'static str {
        "SwapMutator"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Mutator
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;

        for i in 0..chromosome.len() {
            if random_provider::random::<f32>() < self.rate {
                let swap_index = random_provider::gen_range(0..chromosome.len());

                if swap_index == i {
                    continue;
                }

                chromosome.get_genes_mut().swap(i, swap_index);
                mutations += 1;
            }
        }

        mutations
    }
}
