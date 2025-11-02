use crate::{GraphChromosome, Node, Op, TreeChromosome, TreeCrossover, TreeNode};
use radiate_core::{AlterResult, Chromosome, Crossover, random_provider};
use std::fmt::Debug;

const DEFAULT_MAX_SIZE: usize = 10;

#[derive(Clone, Debug)]
pub struct PgmCrossover {
    rate: f32,
    max_size: usize,
}

impl PgmCrossover {
    pub fn new(rate: f32) -> Self {
        Self {
            rate,
            max_size: DEFAULT_MAX_SIZE,
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }
}

impl<T> Crossover<GraphChromosome<Op<T>>> for PgmCrossover
where
    T: Clone + PartialEq,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut GraphChromosome<Op<T>>,
        chrom_two: &mut GraphChromosome<Op<T>>,
        _: f32,
    ) -> AlterResult {
        let one_pgm_indices = chrom_one
            .iter()
            .enumerate()
            .filter_map(|(i, node)| node.value().programs().map(|_| i))
            .collect::<Vec<usize>>();
        let two_pgm_indices = chrom_two
            .iter()
            .enumerate()
            .filter_map(|(i, node)| node.value().programs().map(|_| i))
            .collect::<Vec<usize>>();

        if one_pgm_indices.is_empty() || two_pgm_indices.is_empty() {
            return AlterResult::empty();
        }

        let one_index = random_provider::choose(&one_pgm_indices);
        let two_index = random_provider::choose(&two_pgm_indices);

        let one_value = chrom_one.get_mut(*one_index).value_mut();
        let two_value = chrom_two.get_mut(*two_index).value_mut();

        if let Some(one_progs) = one_value.programs_mut()
            && let Some(two_progs) = two_value.programs_mut()
        {
            return matched_pgm_crossover(one_progs, two_progs, self.max_size);
        }

        AlterResult::empty()
    }
}

impl<T> Crossover<TreeChromosome<Op<T>>> for PgmCrossover
where
    T: Clone + PartialEq,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut TreeChromosome<Op<T>>,
        chrom_two: &mut TreeChromosome<Op<T>>,
        _: f32,
    ) -> AlterResult {
        let swap_one_index = random_provider::range(0..chrom_one.len());
        let swap_two_index = random_provider::range(0..chrom_two.len());

        let one_node = chrom_one.get_mut(swap_one_index);
        let two_node = chrom_two.get_mut(swap_two_index);

        let size_one = one_node.size();
        let size_two = two_node.size();

        let one_sub_node_index = random_provider::range(0..size_one);
        let two_sub_node_index = random_provider::range(0..size_two);

        let one_sub_node = one_node.get_mut(one_sub_node_index);
        let two_sub_node = two_node.get_mut(two_sub_node_index);

        if let (Some(one_sub_node), Some(two_sub_node)) = (one_sub_node, two_sub_node) {
            let one_progs = one_sub_node.value_mut().programs_mut();
            let two_progs = two_sub_node.value_mut().programs_mut();

            match (one_progs, two_progs) {
                (Some(one_progs_mut), Some(two_progs_mut)) => {
                    return matched_pgm_crossover(one_progs_mut, two_progs_mut, self.max_size);
                }
                (Some(one_progs_mut), None) => {
                    let one_rand_prog = random_provider::choose_mut(one_progs_mut);
                    return TreeCrossover::cross_nodes(one_rand_prog, two_sub_node, self.max_size);
                }
                (None, Some(two_progs_mut)) => {
                    let two_rand_prog = random_provider::choose_mut(two_progs_mut);
                    return TreeCrossover::cross_nodes(one_sub_node, two_rand_prog, self.max_size);
                }
                _ => {}
            }
        }

        AlterResult::empty()
    }
}

fn matched_pgm_crossover<T>(
    mut progs_one: &mut Vec<TreeNode<Op<T>>>,
    mut progs_two: &mut Vec<TreeNode<Op<T>>>,
    max_size: usize,
) -> AlterResult
where
    T: Clone,
{
    let one_rand_prog = random_provider::choose_mut(&mut progs_one);
    let two_rand_prog = random_provider::choose_mut(&mut progs_two);

    AlterResult::from(TreeCrossover::cross_nodes(
        one_rand_prog,
        two_rand_prog,
        max_size,
    ))
}
