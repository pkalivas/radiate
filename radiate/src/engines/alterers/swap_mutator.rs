use crate::alter::AlterType;
use crate::engines::genome::genes::gene::Gene;
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
    fn mutate_chromosome(&self, chromosome: &mut C, range: i32) -> i32 {
        let mut mutations = 0;

        for i in 0..chromosome.len() {
            if random_provider::random::<i32>() > range {
                let swap_index = random_provider::gen_range(0..chromosome.len());

                if swap_index == i {
                    continue;
                }

                mutations += 1;

                let curr_gene = chromosome.get_gene(i);
                let swap_gene = chromosome.get_gene(swap_index);

                chromosome.set_gene(i, curr_gene.from_allele(swap_gene.allele()));
            }
        }

        mutations
    }
}
