pub mod graphs;
pub mod mutator;
pub mod node;

pub mod reducers;
pub mod store;
pub mod trees;

pub use graphs::{
    Direction, Graph, GraphArchitect, GraphBuilder, GraphChromosome, GraphCodex, GraphCrossover,
    GraphIterator, GraphMutator, GraphNode, NodeMutate,
};
pub use mutator::OperationMutator;
pub use node::{NodeCell, NodeType};

pub use store::*;

pub use reducers::*;

pub use trees::{
    Tree, TreeBuilder, TreeChromosome, TreeCodex, TreeCrossover, TreeIterator, TreeNode,
};

pub trait Builder {
    type Output;
    fn build(&self) -> Self::Output;
}
