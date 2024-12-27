use crate::node::Node;
use crate::{NodeChromosome, NodeCollectionBuilder, Tree};
use radiate::alter::AlterType;
use radiate::{random_provider, Alter, Chromosome, Valid};
use uuid::Uuid;

pub struct TreeCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub rate: f32,
    pub max_height: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TreeCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub fn new(rate: f32, max_height: usize) -> Self {
        Self {
            rate,
            max_height,
            _marker: std::marker::PhantomData,
        }
    }

    fn level(index: usize, nodes: &[Node<T>]) -> usize {
        nodes[index]
            .incoming
            .iter()
            .map(|i| TreeCrossover::level(*i, nodes))
            .max()
            .unwrap_or(0)
            + 1
    }

    fn depth(index: usize, nodes: &[Node<T>]) -> usize {
        nodes[index]
            .outgoing
            .iter()
            .map(|i| TreeCrossover::depth(*i, nodes))
            .max()
            .unwrap_or(0)
            + 1
    }

    fn can_cross(
        &self,
        one: &[Node<T>],
        two: &[Node<T>],
        one_index: usize,
        two_index: usize,
    ) -> bool {
        if one_index < 1 || two_index < 1 {
            return false;
        }

        let one_depth = TreeCrossover::depth(one_index, one);
        let two_depth = TreeCrossover::depth(two_index, two);

        let one_height = TreeCrossover::level(one_index, one);
        let two_height = TreeCrossover::level(two_index, two);

        one_height + two_depth <= self.max_height && two_height + one_depth <= self.max_height
    }
}

impl<T> Alter<NodeChromosome<T>> for TreeCrossover<T>
where
    T: Clone + PartialEq + Default,
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
        chrom_one: &mut NodeChromosome<T>,
        chrom_two: &mut NodeChromosome<T>,
    ) -> i32 {
        let swap_one_index = random_provider::random::<usize>() % chrom_one.len();
        let swap_two_index = random_provider::random::<usize>() % chrom_two.len();

        if !self.can_cross(
            chrom_one.get_genes(),
            chrom_two.get_genes(),
            swap_one_index,
            swap_two_index,
        ) {
            return 0;
        }

        for node in chrom_one.iter_mut() {
            node.id = Uuid::new_v4();
        }

        for node in chrom_two.iter_mut() {
            node.id = Uuid::new_v4();
        }

        // let tree_one = Tree::new(chrom_one.get_genes().to_vec());
        // let tree_two = Tree::new(chrom_two.get_genes().to_vec());

        // let one_sub_tree = tree_one.sub_tree(swap_one_index);
        // let two_sub_tree = tree_two.sub_tree(swap_two_index);

        // let new_one_tree = NodeCollectionBuilder::default()
        //     .insert(&tree_one)
        //     .replace(&one_sub_tree, &two_sub_tree)
        //     .build();

        // let new_two_tree = NodeCollectionBuilder::default()
        //     .insert(&tree_two)
        //     .replace(&two_sub_tree, &one_sub_tree)
        //     .build();

        // if !new_one_tree.is_valid() || !new_two_tree.is_valid() {
        //     panic!("Invalid tree after crossover.");
        // }

        // chrom_one.nodes = new_one_tree.nodes;
        // chrom_two.nodes = new_two_tree.nodes;

        2
    }
}
