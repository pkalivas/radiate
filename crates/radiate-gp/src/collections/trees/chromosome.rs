use crate::{NodeStore, TreeNode};
use radiate_core::{Chromosome, Valid};
use std::sync::Arc;

type Constraint<N> = Arc<dyn Fn(&N) -> bool>;

#[derive(Clone, Default)]
pub struct TreeChromosome<T> {
    nodes: Vec<TreeNode<T>>,
    store: Option<NodeStore<T>>,
    constraint: Option<Constraint<TreeNode<T>>>,
}

impl<T> TreeChromosome<T> {
    pub fn new(
        nodes: Vec<TreeNode<T>>,
        store: Option<NodeStore<T>>,
        constraint: Option<Constraint<TreeNode<T>>>,
    ) -> Self {
        TreeChromosome {
            nodes,
            store,
            constraint,
        }
    }

    pub fn root(&self) -> &TreeNode<T> {
        &self.nodes[0]
    }

    pub fn root_mut(&mut self) -> &mut TreeNode<T> {
        &mut self.nodes[0]
    }

    pub fn get_store(&self) -> Option<NodeStore<T>> {
        self.store.clone()
    }
}

impl<T> Chromosome for TreeChromosome<T>
where
    T: Clone + PartialEq,
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
