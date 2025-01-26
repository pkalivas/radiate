use crate::collections::trees::TreeBuilder;
use crate::collections::{Tree, TreeChromosome, TreeNode};

use crate::{Builder, Op};
use radiate::{Chromosome, Codex, Genotype};
use std::sync::Arc;

pub struct TreeCodex<T: Clone> {
    builder: TreeBuilder<T>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn new(depth: usize) -> Self {
        TreeCodex {
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

impl<T> Codex<TreeChromosome<T>, Tree<T>> for TreeCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        let root = self.builder.build().take_root().unwrap();

        if let Some(constraint) = &self.constraint {
            if !constraint(&root) {
                panic!("Root node does not meet constraint.");
            }
        }

        let chromosome = TreeChromosome::new(vec![root], self.constraint.clone());
        Genotype::new(vec![chromosome])
    }

    fn decode(&self, genotype: &Genotype<TreeChromosome<T>>) -> Tree<T> {
        let nodes = genotype
            .iter()
            .next()
            .unwrap()
            .iter()
            .collect::<Vec<&TreeNode<T>>>();

        Tree::new((*nodes.first().unwrap()).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ops::Op;
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let codex = TreeCodex::<f32>::new(3)
            .gates(vec![Op::add(), Op::sub(), Op::mul()])
            .leafs(vec![Op::value(1.0), Op::value(2.0)]);

        let genotype = codex.encode();
        let tree = codex.decode(&genotype);

        assert!(tree.root().unwrap().height() == 3);
        assert!(tree.root().is_some());
    }
}
