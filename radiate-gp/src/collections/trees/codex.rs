use crate::NodeStore;
use crate::collections::{Tree, TreeChromosome, TreeNode};
use radiate::{Chromosome, Codex, Genotype};
use std::sync::Arc;

pub struct TreeCodex<T: Clone> {
    depth: usize,
    num_trees: usize,
    store: Option<NodeStore<T>>,
    constraint: Option<Arc<dyn Fn(&TreeNode<T>) -> bool>>,
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
        self.constraint = Some(Arc::new(constraint));
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
                .filter_map(|root| root.map(|node| vec![node]))
                .map(|tree| TreeChromosome::new(tree, Some(store.clone()), self.constraint.clone()))
                .collect::<Vec<TreeChromosome<T>>>();

            if let Some(constraint) = &self.constraint {
                for chromosome in &new_chromosomes {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{NodeType, ops::Op};
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let store = vec![
            (NodeType::Root, vec![Op::add(), Op::sub()]),
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let codex = TreeCodex::single(3, store);

        let genotype = codex.encode();
        let tree = codex.decode(&genotype).first().unwrap().clone();

        assert!(tree.root().unwrap().height() == 3);
        assert!(tree.root().is_some());
    }
}
