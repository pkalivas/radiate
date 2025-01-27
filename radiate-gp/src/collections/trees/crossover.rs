use super::TreeChromosome;

use radiate::engines::genome::*;
use radiate::{random_provider, Alter, AlterAction, Crossover, EngineCompoment};

pub struct TreeCrossover {
    rate: f32,
}

impl TreeCrossover {
    pub fn new(rate: f32) -> Self {
        TreeCrossover { rate }
    }
}

impl EngineCompoment for TreeCrossover {
    fn name(&self) -> &'static str {
        "TreeCrossover"
    }
}

impl<T> Alter<TreeChromosome<T>> for TreeCrossover
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<TreeChromosome<T>> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<T> Crossover<TreeChromosome<T>> for TreeCrossover
where
    T: Clone + PartialEq + Default,
{
    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut TreeChromosome<T>,
        chrom_two: &mut TreeChromosome<T>,
    ) -> i32 {
        let swap_one_index = random_provider::random::<usize>() % chrom_one.len();
        let swap_two_index = random_provider::random::<usize>() % chrom_two.len();

        let one_node = &mut chrom_one.as_mut()[swap_one_index];
        let two_node = &mut chrom_two.as_mut()[swap_two_index];

        let one_size = one_node.size();
        let two_size = two_node.size();

        let one_rand_index = random_provider::random::<usize>() % one_size;
        let two_rand_index = random_provider::random::<usize>() % two_size;

        if one_rand_index < 1 || two_rand_index < 1 {
            return 0;
        }

        one_node.swap_subtrees(two_node, one_rand_index, two_rand_index);

        2
    }
}
