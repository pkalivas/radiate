use crate::{NodeCell, TreeNode};
use radiate::{Chromosome, Valid};
use std::sync::Arc;

type Constraint<N> = Arc<Box<dyn Fn(&N) -> bool>>;

#[derive(Clone, Default)]
pub struct TreeChromosome<C: NodeCell> {
    nodes: Vec<TreeNode<C>>,
    constraint: Option<Constraint<TreeNode<C>>>,
}

impl<C: NodeCell> TreeChromosome<C> {
    pub fn new(nodes: Vec<TreeNode<C>>, constraint: Option<Constraint<TreeNode<C>>>) -> Self {
        TreeChromosome { nodes, constraint }
    }
}

impl<C> Chromosome for TreeChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    type Gene = TreeNode<C>;
}

impl<C> Valid for TreeChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
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

impl<C> AsRef<[TreeNode<C>]> for TreeChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn as_ref(&self) -> &[TreeNode<C>] {
        &self.nodes
    }
}

impl<C> AsMut<[TreeNode<C>]> for TreeChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn as_mut(&mut self) -> &mut [TreeNode<C>] {
        &mut self.nodes
    }
}

impl<C> PartialEq for TreeChromosome<C>
where
    C: PartialEq + NodeCell,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}
