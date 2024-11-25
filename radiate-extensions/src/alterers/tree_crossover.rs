use radiate::{Alterer, Chromosome, Crossover, RandomProvider, Valid};
use uuid::Uuid;

use crate::{Node, NodeCollectionBuilder, Ops, Tree};

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
    pub fn alterer(rate: f32, max_height: usize) -> Alterer<Node<T>, Ops<T>> {
        Alterer::Crossover(Box::new(Self {
            rate,
            max_height,
            _marker: std::marker::PhantomData,
        }))
    }

    fn level(&self, index: usize, nodes: &[Node<T>]) -> usize {
        nodes[index]
            .incoming
            .iter()
            .map(|i| self.level(*i, nodes))
            .max()
            .unwrap_or(0)
            + 1
    }

    fn depth(&self,index: usize, nodes: &[Node<T>]) -> usize {
        nodes[index]
            .outgoing
            .iter()
            .map(|i| self.depth(*i, nodes))
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

        let one_depth = self.depth(one_index, one);
        let two_depth = self.depth(two_index, two);

        let one_height = self.level(one_index, one);
        let two_height = self.level(two_index, two);

        one_height + two_depth <= self.max_height && two_height + one_depth <= self.max_height
    }
}

impl<T> Crossover<Node<T>, Ops<T>> for TreeCrossover<T>
where
    T: Clone + PartialEq + Default,
{
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "Tree Crossover"
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut Chromosome<Node<T>, Ops<T>>,
        chrom_two: &mut Chromosome<Node<T>, Ops<T>>,
    ) -> i32 {
        let swap_one_index = RandomProvider::random::<usize>() % chrom_one.len();
        let swap_two_index = RandomProvider::random::<usize>() % chrom_two.len();

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

        let tree_one = Tree::new(chrom_one.get_genes().to_vec());
        let tree_two = Tree::new(chrom_two.get_genes().to_vec());

        let one_sub_tree = tree_one.sub_tree(swap_one_index);
        let two_sub_tree = tree_two.sub_tree(swap_two_index);

        let new_one_tree = NodeCollectionBuilder::default()
            .insert(&tree_one)
            .replace(&one_sub_tree, &two_sub_tree)
            .build();

        let new_two_tree = NodeCollectionBuilder::default()
            .insert(&tree_two)
            .replace(&two_sub_tree, &one_sub_tree)
            .build();

        if !new_one_tree.is_valid() || !new_two_tree.is_valid() {
            panic!("Invalid tree after crossover.");
        }

        chrom_one.genes = new_one_tree.nodes;
        chrom_two.genes = new_two_tree.nodes;

        2
    }
}
