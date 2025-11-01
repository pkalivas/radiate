use crate::{GraphChromosome, Node, Op, TreeChromosome, TreeCrossover, TreeNode};
use radiate_core::{AlterResult, Chromosome, Crossover, random_provider};

const DEFAULT_MAX_SIZE: usize = 30;

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
            .filter_map(|(i, node)| node.value().programs().map(|progs| (i, progs)))
            .collect::<Vec<(usize, &[TreeNode<Op<T>>])>>();
        let two_pgm_indices = chrom_two
            .iter()
            .enumerate()
            .filter_map(|(i, node)| node.value().programs().map(|progs| (i, progs)))
            .collect::<Vec<(usize, &[TreeNode<Op<T>>])>>();

        if one_pgm_indices.is_empty() || two_pgm_indices.is_empty() {
            return 0.into();
        }

        let (one_index, one_progs) = random_provider::choose(&one_pgm_indices);
        let (two_index, two_progs) = random_provider::choose(&two_pgm_indices);

        let (result, one_copies, two_copies) =
            matched_pgm_crossover(one_progs, two_progs, self.max_size);

        if result.0 > 0 {
            let one_node = chrom_one.get_mut(*one_index);
            let two_node = chrom_two.get_mut(*two_index);

            *one_node.value_mut() = one_node.value().with_programs(one_copies);
            *two_node.value_mut() = two_node.value().with_programs(two_copies);
        }

        result
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

        let one_node = &mut chrom_one.as_mut()[swap_one_index];
        let two_node = &mut chrom_two.as_mut()[swap_two_index];

        let size_one = one_node.size();
        let size_two = two_node.size();

        let one_sub_node_index = random_provider::range(0..size_one);
        let two_sub_node_index = random_provider::range(0..size_two);

        let one_sub_node = one_node.get_mut(one_sub_node_index);
        let two_sub_node = two_node.get_mut(two_sub_node_index);

        if let (Some(one_sub_node), Some(two_sub_node)) = (one_sub_node, two_sub_node) {
            let one_progs = one_sub_node.value().programs();
            let two_progs = two_sub_node.value().programs();

            if let (Some(one_progs), Some(two_progs)) = (one_progs, two_progs) {
                let (count, one_copies, two_copies) =
                    matched_pgm_crossover(one_progs, two_progs, self.max_size);

                (*one_sub_node.value_mut()) = one_sub_node.value().with_programs(one_copies);
                (*two_sub_node.value_mut()) = two_sub_node.value().with_programs(two_copies);

                return count.into();
            } else if let Some(one_progs) = one_progs {
                let mut one_copies = one_progs
                    .iter()
                    .map(|program| program.clone())
                    .collect::<Vec<TreeNode<Op<T>>>>();

                let one_rand_prog = random_provider::choose_mut(&mut one_copies);
                let count = TreeCrossover::cross_nodes(one_rand_prog, two_sub_node, self.max_size);

                (*one_sub_node.value_mut()) = one_sub_node.value().with_programs(one_copies);

                return count.into();
            } else if let Some(two_progs) = two_progs {
                let mut two_copies = two_progs
                    .iter()
                    .map(|program| program.clone())
                    .collect::<Vec<TreeNode<Op<T>>>>();

                let two_rand_prog = random_provider::choose_mut(&mut two_copies);
                let count = TreeCrossover::cross_nodes(one_sub_node, two_rand_prog, self.max_size);

                (*two_sub_node.value_mut()) = two_sub_node.value().with_programs(two_copies);

                return count.into();
            }
        }

        0.into()
    }
}

fn matched_pgm_crossover<T>(
    progs_one: &[TreeNode<Op<T>>],
    progs_two: &[TreeNode<Op<T>>],
    max_size: usize,
) -> (AlterResult, Vec<TreeNode<Op<T>>>, Vec<TreeNode<Op<T>>>)
where
    T: Clone,
{
    let mut one_copies = progs_one
        .iter()
        .map(|program| program.clone())
        .collect::<Vec<TreeNode<Op<T>>>>();

    let mut two_copies = progs_two
        .iter()
        .map(|program| program.clone())
        .collect::<Vec<TreeNode<Op<T>>>>();

    let one_rand_prog = random_provider::choose_mut(&mut one_copies);
    let two_rand_prog = random_provider::choose_mut(&mut two_copies);

    let count = TreeCrossover::cross_nodes(one_rand_prog, two_rand_prog, max_size);

    (count, one_copies, two_copies)
}
