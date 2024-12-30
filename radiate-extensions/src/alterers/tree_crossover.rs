use crate::collections::{NodeChrom, TreeNode};
use radiate::alter::AlterType;
use radiate::{random_provider, Alter, Chromosome};
use std::fmt::Debug;

pub struct TreeCrossover {
    pub rate: f32,
}

impl TreeCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<T> Alter<NodeChrom<TreeNode<T>>> for TreeCrossover
where
    T: Clone + PartialEq + Default + Debug,
{
    fn name(&self) -> &'static str {
        "Tree Crossover"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut NodeChrom<TreeNode<T>>,
        chrom_two: &mut NodeChrom<TreeNode<T>>,
    ) -> i32 {
        let swap_one_index = random_provider::random::<usize>() % chrom_one.len();
        let swap_two_index = random_provider::random::<usize>() % chrom_two.len();

        let one_node = &mut chrom_one.get_genes_mut()[swap_one_index];
        let mut two_node = &mut chrom_two.get_genes_mut()[swap_two_index];

        let one_size = one_node.size();
        let two_size = two_node.size();

        let one_rand_index = random_provider::random::<usize>() % one_size;
        let two_rand_index = random_provider::random::<usize>() % two_size;

        if one_rand_index < 1 || two_rand_index < 1 {
            return 0;
        }

        one_node.swap_subtrees(&mut two_node, one_rand_index, two_rand_index);

        2
    }
}
