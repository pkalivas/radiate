use crate::NodeSet;
use crate::collections::{Tree, TreeChromosome};
use radiate_core::{Codec, Genotype};

#[derive(Clone)]
pub struct TreeCodec<T: Clone, D = Vec<Tree<T>>> {
    depth: usize,
    num_trees: usize,
    store: Option<NodeSet<T>>,
    max_size: Option<u32>,
    _marker: std::marker::PhantomData<D>,
}

impl<T: Clone + Default> TreeCodec<T> {
    pub fn single(depth: usize, store: impl Into<NodeSet<T>>) -> TreeCodec<T, Tree<T>> {
        TreeCodec {
            depth,
            num_trees: 1,
            store: Some(store.into()),
            max_size: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn multi_root(
        depth: usize,
        num_trees: usize,
        store: impl Into<NodeSet<T>>,
    ) -> TreeCodec<T, Vec<Tree<T>>> {
        TreeCodec {
            depth,
            num_trees,
            store: Some(store.into()),
            max_size: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, D> TreeCodec<T, D> {
    pub fn max_size(mut self, max_size: u32) -> Self {
        self.max_size = Some(max_size);
        self
    }
}

impl<T> Codec<TreeChromosome<T>, Vec<Tree<T>>> for TreeCodec<T, Vec<Tree<T>>>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        if let Some(store) = &self.store {
            let new_chromosomes = (0..self.num_trees)
                .map(|_| Tree::with_depth(self.depth, store))
                .filter_map(|tree| tree.take_root())
                .map(|root| TreeChromosome::new(root, Some(store.clone()), self.max_size.clone()))
                .collect::<Vec<TreeChromosome<T>>>();

            if new_chromosomes.len() != self.num_trees {
                return Genotype::default();
            }

            return Genotype::new(new_chromosomes);
        }

        Genotype::default()
    }

    fn decode(&self, genotype: &Genotype<TreeChromosome<T>>) -> Vec<Tree<T>> {
        genotype
            .iter()
            .map(|chromosome| Tree::new(chromosome.root().clone()))
            .collect()
    }
}

impl<T> Codec<TreeChromosome<T>, Tree<T>> for TreeCodec<T, Tree<T>>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        if let Some(store) = &self.store {
            let new_chromosome = Tree::with_depth(self.depth, store)
                .take_root()
                .map(|root| TreeChromosome::new(root, Some(store.clone()), self.max_size.clone()));

            return match new_chromosome {
                Some(chrom) => Genotype::from(chrom),
                None => Genotype::default(),
            };
        }

        Genotype::new(vec![])
    }

    fn decode(&self, genotype: &Genotype<TreeChromosome<T>>) -> Tree<T> {
        genotype
            .iter()
            .next()
            .map(|chromosome| Tree::new(chromosome.root().clone()))
            .unwrap_or_default()
    }
}

unsafe impl<T: Send + Clone> Send for TreeCodec<T> {}
unsafe impl<T: Sync + Clone> Sync for TreeCodec<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeType, ops::Op};
    use radiate_core::codecs::Codec;

    #[test]
    fn test_tree_codec() {
        let store = vec![
            (NodeType::Root, vec![Op::add(), Op::sub()]),
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::one(), Op::two()]),
        ];
        let codec = TreeCodec::<Op<f32>>::single(3, store);

        let genotype = codec.encode();
        let tree = codec.decode(&genotype);

        assert_eq!(tree.root().map(|root| root.height()), Some(3));
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_tree_codec_multi() {
        let store = vec![
            (NodeType::Root, vec![Op::add(), Op::sub()]),
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::one(), Op::two()]),
        ];
        let codec = TreeCodec::<Op<f32>>::multi_root(3, 2, store);

        let genotype = codec.encode();
        let trees = codec.decode(&genotype);

        assert_eq!(trees.len(), 2);
        assert_eq!(trees[0].root().map(|root| root.height()), Some(3));
        assert_eq!(trees[1].root().map(|root| root.height()), Some(3));
        assert!(trees[0].root().is_some());
        assert!(trees[1].root().is_some());
    }
}
