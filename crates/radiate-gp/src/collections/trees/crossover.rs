use super::TreeChromosome;
use radiate_core::genome::*;
use radiate_core::{AlterResult, Crossover, random_provider};

#[derive(Clone, Debug)]
pub struct TreeCrossover {
    rate: f32,
}

impl TreeCrossover {
    pub fn new(rate: f32) -> Self {
        TreeCrossover { rate }
    }
}

impl<T> Crossover<TreeChromosome<T>> for TreeCrossover
where
    T: Clone + PartialEq,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut TreeChromosome<T>,
        chrom_two: &mut TreeChromosome<T>,
        _: f32,
    ) -> AlterResult {
        let swap_one_index = random_provider::range(0..chrom_one.len());
        let swap_two_index = random_provider::range(0..chrom_two.len());

        let one_node = &mut chrom_one.as_mut()[swap_one_index];
        let two_node = &mut chrom_two.as_mut()[swap_two_index];

        let one_size = one_node.size();
        let two_size = two_node.size();

        let one_rand_index = random_provider::range(0..one_size);
        let two_rand_index = random_provider::range(0..two_size);

        if one_rand_index < 1 || two_rand_index < 1 {
            return 0.into();
        }

        one_node.swap_subtrees(two_node, one_rand_index, two_rand_index);

        2.into()
    }
}
