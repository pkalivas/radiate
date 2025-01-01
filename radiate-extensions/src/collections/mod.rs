pub mod crossover;
pub mod graphs;
pub mod mutator;
pub mod node;
pub mod reducers;
pub mod store;
pub mod trees;

pub use crossover::{GraphCrossover, TreeCrossover};
pub use graphs::{Direction, Graph, GraphChromosome, GraphCodex, GraphNode};
pub use mutator::{GraphMutator, NodeMutate, OperationMutator};
pub use node::{Node, NodeCell, NodeType};
pub use store::*;

pub use reducers::*;

pub use trees::{Tree, TreeBuilder, TreeChromosome, TreeCodex, TreeIterator, TreeNode};

pub trait Builder {
    type Output;
    fn build(&self) -> Self::Output;
}
