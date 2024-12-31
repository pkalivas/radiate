use crate::TreeNode;
use radiate::{Chromosome, Valid};
use std::sync::Arc;

type Constraint<N> = Arc<Box<dyn Fn(&N) -> bool>>;

#[derive(Clone, Default)]
pub struct TreeChromosome<T> {
    nodes: Vec<TreeNode<T>>,
    constraint: Option<Constraint<TreeNode<T>>>,
}

impl<T> TreeChromosome<T> {
    pub fn new(nodes: Vec<TreeNode<T>>) -> Self {
        TreeChromosome {
            nodes,
            constraint: None,
        }
    }

    pub fn with_constraint(
        nodes: Vec<TreeNode<T>>,
        constraint: Option<Constraint<TreeNode<T>>>,
    ) -> Self {
        TreeChromosome { nodes, constraint }
    }
}

impl<T> Chromosome for TreeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Gene = TreeNode<T>;

    // fn get_genes(&self) -> &[TreeNode<T>] {
    //     &self.nodes
    // }

    // fn get_genes_mut(&mut self) -> &mut [TreeNode<T>] {
    //     &mut self.nodes
    // }
}

impl<T> Valid for TreeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
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

impl<T> AsRef<[TreeNode<T>]> for TreeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn as_ref(&self) -> &[TreeNode<T>] {
        &self.nodes
    }
}

impl<T> AsMut<[TreeNode<T>]> for TreeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn as_mut(&mut self) -> &mut [TreeNode<T>] {
        &mut self.nodes
    }
}

impl<T> PartialEq for TreeChromosome<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}
