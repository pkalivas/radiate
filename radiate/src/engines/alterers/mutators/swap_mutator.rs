use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::gene::Gene;
use crate::RandomProvider;

use super::mutate::Mutate;

pub struct SwapMutator {
    rate: f32,
}

impl SwapMutator {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<G: Gene<G, A>, A> Mutate<G, A> for SwapMutator {
    fn mutate_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "SwapMutator"
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut Chromosome<G, A>, range: i32) -> i32 {
        let mut mutations = 0;

        for i in 0..chromosome.len() {
            if RandomProvider::random::<i32>() > range {
                let swap_index = RandomProvider::gen_range(0..chromosome.len());

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
