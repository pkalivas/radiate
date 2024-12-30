pub mod crossover;
pub mod factory;
pub mod graphs;
pub mod mutator;
pub mod reducers;
pub mod trees;

use crate::ops::Operation;

pub use crossover::{GraphCrossover, NodeCrossover, TreeCrossover};
pub use factory::*;
pub use graphs::{Direction, Graph, GraphChromosome, GraphCodex, GraphNode, NodeType};
pub use mutator::{GraphMutator, NodeMutate, OperationMutator};

pub use reducers::*;

pub use trees::{Tree, TreeBuilder, TreeChromosome, TreeCodex, TreeIterator, TreeNode};

pub trait Builder {
    type Output;
    fn build(&self) -> Self::Output;
}
