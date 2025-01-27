use crate::{Op, TreeNode};
use radiate::{Chromosome, Valid};
use std::sync::{Arc, RwLock};

type Constraint<N> = Arc<Box<dyn Fn(&N) -> bool>>;

#[derive(Clone, Default)]
pub struct TreeChromosome<T> {
    nodes: Vec<TreeNode<T>>,
    gates: Arc<RwLock<Vec<Op<T>>>>,
    leafs: Arc<RwLock<Vec<Op<T>>>>,
    constraint: Option<Constraint<TreeNode<T>>>,
}

impl<T> TreeChromosome<T> {
    pub fn new(
        nodes: Vec<TreeNode<T>>,
        gates: Arc<RwLock<Vec<Op<T>>>>,
        leafs: Arc<RwLock<Vec<Op<T>>>>,
        constraint: Option<Constraint<TreeNode<T>>>,
    ) -> Self {
        TreeChromosome {
            nodes,
            gates,
            leafs,
            constraint,
        }
    }

    pub fn root(&self) -> &TreeNode<T> {
        &self.nodes[0]
    }

    pub fn root_mut(&mut self) -> &mut TreeNode<T> {
        &mut self.nodes[0]
    }

    pub fn get_leafs(&self) -> Arc<RwLock<Vec<Op<T>>>> {
        Arc::clone(&self.leafs)
    }

    pub fn get_gates(&self) -> Arc<RwLock<Vec<Op<T>>>> {
        Arc::clone(&self.gates)
    }
}

impl<T> Chromosome for TreeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Gene = TreeNode<T>;
}

impl<T> Valid for TreeChromosome<T> {
    fn is_valid(&self) -> bool {
        for gene in &self.nodes {
            if let Some(constraint) = &self.constraint {
                if !constraint(gene) {
                    return false;
                }
            } else if !gene.is_valid() {
                return false;
            }
        }

        true
    }
}

impl<T> AsRef<[TreeNode<T>]> for TreeChromosome<T> {
    fn as_ref(&self) -> &[TreeNode<T>] {
        &self.nodes
    }
}

impl<T> AsMut<[TreeNode<T>]> for TreeChromosome<T> {
    fn as_mut(&mut self) -> &mut [TreeNode<T>] {
        &mut self.nodes
    }
}

impl<T: PartialEq> PartialEq for TreeChromosome<T> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}
