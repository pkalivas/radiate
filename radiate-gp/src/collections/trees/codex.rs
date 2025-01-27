use crate::collections::trees::TreeBuilder;
use crate::collections::{Tree, TreeChromosome, TreeNode};
use crate::{Builder, Op};
use radiate::{Codex, Genotype};
use std::sync::Arc;

pub struct TreeCodex<T: Clone> {
    num_trees: usize,
    builder: TreeBuilder<T>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn new(num_trees: usize, depth: usize) -> Self {
        TreeCodex {
            num_trees,
            builder: TreeBuilder::new(depth),
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

    pub fn gates(mut self, gates: Vec<Op<T>>) -> Self {
        self.builder = self.builder.with_gates(gates);
        self
    }

    pub fn leafs(mut self, leafs: Vec<Op<T>>) -> Self {
        self.builder = self.builder.with_leafs(leafs);
        self
    }
}

impl<T> Codex<TreeChromosome<T>, Vec<Tree<T>>> for TreeCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        let trees = (0..self.num_trees)
            .map(|_| self.builder.build().take_root().unwrap())
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
                    TreeChromosome::new(
                        vec![tree],
                        self.builder.get_gates(),
                        self.builder.get_leafs(),
                        self.constraint.clone(),
                    )
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

impl<T: Clone + Default> Default for TreeCodex<T> {
    fn default() -> Self {
        TreeCodex::new(1, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ops::Op;
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let codex = TreeCodex::<f32>::new(1, 3)
            .gates(vec![Op::add(), Op::sub(), Op::mul()])
            .leafs(vec![Op::value(1.0), Op::value(2.0)]);

        let genotype = codex.encode();
        let tree = codex.decode(&genotype).first().unwrap().clone();

        assert!(tree.root().unwrap().height() == 3);
        assert!(tree.root().is_some());
    }
}
