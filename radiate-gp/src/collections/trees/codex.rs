use crate::collections::{Tree, TreeChromosome, TreeNode};
use crate::NodeStore;
use radiate::{Codex, Genotype};
use std::sync::Arc;

pub struct TreeCodex<T: Clone> {
    depth: usize,
    store: Option<NodeStore<T>>,
    templates: Option<Vec<Tree<T>>>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn single(depth: usize, store: impl Into<NodeStore<T>>) -> Self {
        let store = store.into();
        let tree = Tree::with_depth(depth, store.clone());

        TreeCodex {
            depth,
            store: Some(store),
            templates: Some(vec![tree]),
            constraint: None,
        }
    }

    pub fn multi_root(depth: usize, num_trees: usize, store: impl Into<NodeStore<T>>) -> Self {
        let store = store.into();
        let trees = (0..num_trees)
            .map(|_| Tree::with_depth(depth, store.clone()))
            .collect::<Vec<_>>();

        TreeCodex {
            depth,
            store: Some(store),
            templates: Some(trees),
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
        let trees = (0..self.templates.as_ref().unwrap().len())
            .map(|_| {
                Tree::with_depth(self.depth, self.store.clone().unwrap())
                    .take_root()
                    .unwrap()
            })
            .collect::<Vec<_>>();

        if let Some(constraint) = &self.constraint {
            for tree in &trees {
                if !constraint(tree) {
                    panic!("Root node does not meet constraint.");
                }
            }
        }

        Genotype::new(
            trees
                .into_iter()
                .map(|tree| {
                    TreeChromosome::new(vec![tree], self.store.clone(), self.constraint.clone())
                })
                .collect(),
        )
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
