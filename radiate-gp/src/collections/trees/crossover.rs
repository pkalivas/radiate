use crate::NodeCell;

use super::TreeChromosome;

use radiate::engines::genome::*;
use radiate::{random_provider, Alter, AlterAction, Crossover, EngineCompoment};

pub struct TreeCrossover {
    pub rate: f32,
}

impl TreeCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl EngineCompoment for TreeCrossover {
    fn name(&self) -> &'static str {
        "TreeCrossover"
    }
}

impl<C> Alter<TreeChromosome<C>> for TreeCrossover
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<TreeChromosome<C>> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<C> Crossover<TreeChromosome<C>> for TreeCrossover
where
    C: Clone + PartialEq + Default + NodeCell,
{
    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut TreeChromosome<C>,
        chrom_two: &mut TreeChromosome<C>,
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
