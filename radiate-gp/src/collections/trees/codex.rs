use crate::collections::{Tree, TreeChromosome, TreeNode};
use crate::NodeStore;
use radiate::{Chromosome, Codex, Genotype};
use std::sync::Arc;

pub struct TreeCodex<T: Clone> {
    depth: usize,
    num_trees: usize,
    store: Option<NodeStore<T>>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn single(depth: usize, store: impl Into<NodeStore<T>>) -> Self {
        TreeCodex {
            depth,
            num_trees: 1,
            store: Some(store.into()),
            constraint: None,
        }
    }

    pub fn multi_root(depth: usize, num_trees: usize, store: impl Into<NodeStore<T>>) -> Self {
        TreeCodex {
            depth,
            num_trees,
            store: Some(store.into()),
            constraint: None,
        }
    }

    pub fn constraint<F>(mut self, constraint: F) -> Self
    where
        F: Fn(&TreeNode<T>) -> bool + 'static,
    {
        self.constraint = Some(Arc::new(Box::new(constraint)));
        self
    }
}

impl<T> Codex<TreeChromosome<T>, Vec<Tree<T>>> for TreeCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        if let Some(store) = &self.store {
            let new_chromosomes = (0..self.num_trees)
                .map(|_| Tree::with_depth(self.depth, store).take_root())
                .filter_map(|tree| tree.map(|tree| vec![tree]))
                .map(|tree| TreeChromosome::new(tree, Some(store.clone()), self.constraint.clone()))
                .collect::<Vec<TreeChromosome<T>>>();

            for chromosome in &new_chromosomes {
                if let Some(constraint) = &self.constraint {
                    for tree in chromosome.iter() {
                        if !constraint(tree) {
                            panic!("Root node does not meet constraint.");
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{ops::Op, NodeType};
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let store = vec![
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::value(1.0), Op::value(2.0)]),
        ];
        let codex = TreeCodex::single(3, store);

        let genotype = codex.encode();
        let tree = codex.decode(&genotype).first().unwrap().clone();

        assert!(tree.root().unwrap().height() == 3);
        assert!(tree.root().is_some());
    }
}
