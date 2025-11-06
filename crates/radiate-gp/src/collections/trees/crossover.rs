use super::TreeChromosome;
use crate::TreeNode;
use radiate_core::{AlterResult, Crossover, random_provider};
use radiate_core::{genome::*, metric};

const DEFAULT_MAX_SIZE: usize = 30;
const MAX_ATTEMPTS: usize = 3;
const TN_X_ATTEMPTS: &str = "tn_x_att";

#[derive(Clone, Debug)]
pub struct TreeCrossover {
    rate: f32,
    max_size: usize,
}

impl TreeCrossover {
    pub fn new(rate: f32) -> Self {
        TreeCrossover {
            rate,
            max_size: DEFAULT_MAX_SIZE,
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn cross_nodes<T>(
        node_one: &mut TreeNode<T>,
        node_two: &mut TreeNode<T>,
        max_size: usize,
    ) -> AlterResult {
        let one_size = node_one.size();
        let two_size = node_two.size();

        if one_size == 1 || two_size == 1 {
            return AlterResult::empty();
        }

        let mut attempts = 0;
        while attempts < MAX_ATTEMPTS {
            let one_rand_index = random_provider::range(1..one_size);
            let two_rand_index = random_provider::range(1..two_size);

            let one_sub_node = node_one.get_mut(one_rand_index);
            let two_sub_node = node_two.get_mut(two_rand_index);

            if let (Some(one_sub_node), Some(two_sub_node)) = (one_sub_node, two_sub_node) {
                let one_sub_size = one_sub_node.size();
                let two_sub_size = two_sub_node.size();

                let one_crossover_size = one_size - one_sub_size + two_sub_size;
                let two_crossover_size = two_size - two_sub_size + one_sub_size;

                if one_crossover_size <= max_size && two_crossover_size <= max_size {
                    std::mem::swap(one_sub_node, two_sub_node);
                    return AlterResult::from((2, metric!(TN_X_ATTEMPTS, attempts + 1)));
                }
            }

            attempts += 1;
        }

        AlterResult::empty()
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

        let one_node = chrom_one.get_mut(swap_one_index);
        let two_node = chrom_two.get_mut(swap_two_index);

        Self::cross_nodes(one_node, two_node, self.max_size)
    }
}
