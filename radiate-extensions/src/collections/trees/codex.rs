use crate::collections::trees::TreeBuilder;
use crate::collections::{Tree, TreeChromosome, TreeNode};

use crate::NodeCell;
use radiate::{Chromosome, Codex, Genotype};
use std::sync::Arc;

pub struct TreeCodex<C: Clone + NodeCell> {
    architect: TreeBuilder<C>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<C>) -> bool>>>,
}

impl<C: NodeCell + Clone + Default> TreeCodex<C> {
    pub fn new(depth: usize) -> Self {
        TreeCodex {
            architect: TreeBuilder::new(depth),
            constraint: None,
        }
    }

    pub fn constraint<F>(mut self, constraint: F) -> Self
    where
        F: Fn(&TreeNode<C>) -> bool + 'static,
    {
        self.constraint = Some(Arc::new(Box::new(constraint)));
        self
    }

    pub fn gates(mut self, gates: Vec<C>) -> Self {
        self.architect = self.architect.with_gates(gates);
        self
    }

    pub fn leafs(mut self, leafs: Vec<C>) -> Self {
        self.architect = self.architect.with_leafs(leafs);
        self
    }
}

impl<C> Codex<TreeChromosome<C>, Tree<C>> for TreeCodex<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn encode(&self) -> Genotype<TreeChromosome<C>> {
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

    fn decode(&self, genotype: &Genotype<TreeChromosome<C>>) -> Tree<C> {
        let nodes = genotype
            .iter()
            .next()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TreeNode<C>>>()
            .first()
            .unwrap()
            .to_owned();

        Tree::new(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ops::Operation;
    use radiate::engines::codexes::Codex;

    #[test]
    fn test_tree_codex() {
        let codex = TreeCodex::<Operation<f32>>::new(3)
            .gates(vec![Operation::add(), Operation::sub(), Operation::mul()])
            .leafs(vec![Operation::value(1.0), Operation::value(2.0)]);
        let genotype = codex.encode();
        let tree = codex.decode(&genotype);

        assert!(tree.root().is_some());
    }
}
