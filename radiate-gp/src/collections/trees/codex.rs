use crate::collections::{Tree, TreeChromosome, TreeNode};
use crate::NodeStore;
use radiate::{Chromosome, Codex, Genotype};
use std::sync::Arc;

type Constraint<N> = Arc<dyn Fn(&N) -> bool>;

pub struct TreeCodex<T: Clone, D = Vec<Tree<T>>> {
    depth: usize,
    num_trees: usize,
    store: Option<NodeStore<T>>,
    constraint: Option<Constraint<TreeNode<T>>>,
    _marker: std::marker::PhantomData<D>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn single(depth: usize, store: impl Into<NodeStore<T>>) -> TreeCodex<T, Tree<T>> {
        TreeCodex {
            depth,
            num_trees: 1,
            store: Some(store.into()),
            constraint: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn multi_root(
        depth: usize,
        num_trees: usize,
        store: impl Into<NodeStore<T>>,
    ) -> TreeCodex<T, Vec<Tree<T>>> {
        TreeCodex {
            depth,
            num_trees,
            store: Some(store.into()),
            constraint: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, D> TreeCodex<T, D> {
    pub fn constraint<F>(mut self, constraint: F) -> Self
    where
        F: Fn(&TreeNode<T>) -> bool + 'static,
    {
        self.constraint = Some(Arc::new(constraint));
        self
    }
}

impl<T> Codex<TreeChromosome<T>, Vec<Tree<T>>> for TreeCodex<T, Vec<Tree<T>>>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        if let Some(store) = &self.store {
            let new_chromosomes = (0..self.num_trees)
                .map(|_| Tree::with_depth(self.depth, store).take_root())
                .filter_map(|root| root.map(|node| vec![node]))
                .map(|tree| TreeChromosome::new(tree, Some(store.clone()), self.constraint.clone()))
                .collect::<Vec<TreeChromosome<T>>>();

            if let Some(constraint) = &self.constraint {
                for chromosome in new_chromosomes.iter() {
                    for node in chromosome.iter() {
                        if !constraint(node) {
                            panic!("TreeCodex.encode() - Root node does not meet constraint.");
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

impl<T> Codex<TreeChromosome<T>, Tree<T>> for TreeCodex<T, Tree<T>>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        if let Some(store) = &self.store {
            let new_chromosome = Tree::with_depth(self.depth, store)
                .take_root()
                .map(|root| vec![root])
                .map(|tree| TreeChromosome::new(tree, Some(store.clone()), self.constraint.clone()))
                .unwrap_or_else(|| TreeChromosome::new(vec![], None, None));

            if let Some(constraint) = &self.constraint {
                if !constraint(new_chromosome.root()) {
                    panic!("TreeCodex.encode() - Root node does not meet constraint.");
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
    use crate::{ops::Op, NodeType};
    use radiate::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let store = vec![
            (NodeType::Root, vec![Op::add(), Op::sub()]),
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let codex = TreeCodex::single(3, store);

        let genotype = codex.encode();
        let tree = codex.decode(&genotype);

        assert_eq!(tree.root().map(|root| root.height()), Some(3));
        assert!(tree.root().is_some());
    }
}
