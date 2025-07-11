use crate::NodeStore;
use crate::collections::{Tree, TreeChromosome, TreeNode};
use radiate_core::{Chromosome, Codec, Genotype};
use std::sync::Arc;

type Constraint<N> = Arc<dyn Fn(&N) -> bool>;

#[derive(Clone)]
pub struct TreeCodec<T: Clone, D = Vec<Tree<T>>> {
    depth: usize,
    num_trees: usize,
    store: Option<NodeStore<T>>,
    constraint: Option<Constraint<TreeNode<T>>>,
    template: Option<Tree<T>>,
    _marker: std::marker::PhantomData<D>,
}

impl<T: Clone + Default> TreeCodec<T> {
    pub fn single(depth: usize, store: impl Into<NodeStore<T>>) -> TreeCodec<T, Tree<T>> {
        TreeCodec {
            depth,
            num_trees: 1,
            store: Some(store.into()),
            constraint: None,
            template: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn multi_root(
        depth: usize,
        num_trees: usize,
        store: impl Into<NodeStore<T>>,
    ) -> TreeCodec<T, Vec<Tree<T>>> {
        TreeCodec {
            depth,
            num_trees,
            store: Some(store.into()),
            constraint: None,
            template: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, D> TreeCodec<T, D> {
    pub fn constraint<F>(mut self, constraint: F) -> Self
    where
        F: Fn(&TreeNode<T>) -> bool + 'static,
    {
        self.constraint = Some(Arc::new(constraint));
        self
    }

    pub fn with_tree(mut self, template: impl Into<Tree<T>>) -> Self {
        self.template = Some(template.into());
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
                .map(|_| match self.template.as_ref() {
                    Some(template) => template.clone(),
                    None => Tree::with_depth(self.depth, store),
                })
                .filter_map(|tree| tree.take_root().map(|root| vec![root]))
                .map(|node| TreeChromosome::new(node, Some(store.clone()), self.constraint.clone()))
                .collect::<Vec<TreeChromosome<T>>>();

            if let Some(constraint) = self.constraint.as_ref() {
                for chromosome in new_chromosomes.iter() {
                    for node in chromosome.iter() {
                        if !constraint(node) {
                            panic!("TreeCodec.encode() - Root node does not meet constraint.");
                        }
                    }
                }
            }

            return Genotype::new(new_chromosomes);
        }

        Genotype::new(vec![])
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
            let tree = match self.template.as_ref() {
                Some(template) => template.clone(),
                None => Tree::with_depth(self.depth, store),
            };

            let new_chromosome = tree
                .take_root()
                .map(|root| vec![root])
                .map(|tree| TreeChromosome::new(tree, Some(store.clone()), self.constraint.clone()))
                .unwrap_or_else(|| TreeChromosome::new(vec![], None, self.constraint.clone()));

            if let Some(constraint) = &self.constraint {
                if !constraint(new_chromosome.root()) {
                    panic!("TreeCodec.encode() - Root node does not meet constraint.");
                }
            }

            return Genotype::new(vec![new_chromosome]);
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
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let codec = TreeCodec::single(3, store);

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
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let codec = TreeCodec::multi_root(3, 2, store);

        let genotype = codec.encode();
        let trees = codec.decode(&genotype);

        assert_eq!(trees.len(), 2);
        assert_eq!(trees[0].root().map(|root| root.height()), Some(3));
        assert_eq!(trees[1].root().map(|root| root.height()), Some(3));
        assert!(trees[0].root().is_some());
        assert!(trees[1].root().is_some());
    }
}
