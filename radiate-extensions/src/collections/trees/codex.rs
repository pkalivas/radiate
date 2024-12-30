use crate::collections::trees::TreeBuilder;
use crate::collections::{Tree, TreeChromosome, TreeNode};
use crate::ops::Operation;
use radiate::{Chromosome, Codex, Genotype};
use std::sync::Arc;

pub struct TreeCodex<T: Clone> {
    architect: TreeBuilder<T>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T: Clone + Default> TreeCodex<T> {
    pub fn new(depth: usize) -> Self {
        TreeCodex {
            architect: TreeBuilder::new(depth),
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

    pub fn gates(mut self, gates: Vec<Operation<T>>) -> Self {
        self.architect = self.architect.with_gates(gates);
        self
    }

    pub fn leafs(mut self, leafs: Vec<Operation<T>>) -> Self {
        self.architect = self.architect.with_leafs(leafs);
        self
    }
}

impl<T> Codex<TreeChromosome<T>, Tree<T>> for TreeCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<TreeChromosome<T>> {
        let root = self.architect.build().root().take().unwrap().to_owned();

        if let Some(constraint) = &self.constraint {
            if !constraint(&root) {
                panic!("Root node does not meet constraint.");
            }
        }

        Genotype {
            chromosomes: vec![TreeChromosome::with_constraint(
                vec![root],
                self.constraint.clone(),
            )],
        }
    }

    fn decode(&self, genotype: &Genotype<TreeChromosome<T>>) -> Tree<T> {
        let nodes = genotype
            .iter()
            .next()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TreeNode<T>>>()
            .first()
            .unwrap()
            .to_owned();

        Tree::new(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::operation;
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let codex = TreeCodex::<f32>::new(3)
            .gates(vec![operation::add(), operation::sub(), operation::mul()])
            .leafs(vec![operation::value(1.0), operation::value(2.0)]);
        let genotype = codex.encode();
        let tree = codex.decode(&genotype);

        assert!(tree.root().is_some());
    }
}
