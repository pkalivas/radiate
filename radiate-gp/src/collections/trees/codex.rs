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

        Genotype::new(vec![TreeChromosome::new(
            vec![root],
            self.builder.get_gates(),
            self.builder.get_leafs(),
            self.constraint.clone(),
        )])
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

#[derive(Clone, PartialEq, Debug)]
pub struct ProgramTree {
    pub trees: Option<Vec<Tree<f32>>>,
}

pub struct ProgramTreeCodex {
    num_trees: usize,
    builder: TreeBuilder<f32>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<f32>) -> bool>>>,
}

impl ProgramTreeCodex {
    pub fn new(depth: usize, num_trees: usize) -> Self {
        ProgramTreeCodex {
            num_trees,
            builder: TreeBuilder::new(depth),
            constraint: None,
        }
    }

    pub fn constraint<F>(mut self, constraint: F) -> Self
    where
        F: Fn(&TreeNode<f32>) -> bool + 'static,
    {
        self.constraint = Some(Arc::new(Box::new(constraint)));
        self
    }

    pub fn gates(mut self, gates: Vec<Op<f32>>) -> Self {
        self.builder = self.builder.with_gates(gates);
        self
    }

    pub fn leafs(mut self, leafs: Vec<Op<f32>>) -> Self {
        self.builder = self.builder.with_leafs(leafs);
        self
    }
}

impl Codex<TreeChromosome<f32>, ProgramTree> for ProgramTreeCodex {
    fn encode(&self) -> Genotype<TreeChromosome<f32>> {
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

    fn decode(&self, genotype: &Genotype<TreeChromosome<f32>>) -> ProgramTree {
        let trees = genotype
            .iter()
            .map(|chromosome| {
                let nodes = chromosome.iter().collect::<Vec<&TreeNode<f32>>>();
                Tree::new((*nodes.first().unwrap()).clone())
            })
            .collect();

        ProgramTree { trees: Some(trees) }
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
