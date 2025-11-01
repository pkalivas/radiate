pub mod dot;
pub mod eval;
pub mod factory;
pub mod format;
pub mod graphs;
pub mod node;
pub mod store;
pub mod trees;

pub use dot::ToDot;
pub use eval::{Eval, EvalMut};
pub use factory::*;
pub use format::*;
pub use graphs::{
    Direction, Graph, GraphAggregate, GraphChromosome, GraphCodec, GraphCrossover, GraphEvaluator,
    GraphIterator, GraphMutator, GraphNode, GraphNodeId, GraphReplacement, NeatDistance,
};
pub use node::{Node, NodeType};
pub use store::{NodeStore, NodeValue};
pub use trees::{
    HoistMutator, Tree, TreeChromosome, TreeCodec, TreeCrossover, TreeIterator, TreeNode,
    TreeRewriter, TreeRewriterRule,
};

pub trait TopologicalMapper<T, U> {
    type OutputNode: Node<Value = U>;

    fn map<F>(&self, mapper_fn: F) -> Self::OutputNode
    where
        F: Fn(&T) -> U;
}

impl<T, U> TopologicalMapper<T, U> for TreeNode<T> {
    type OutputNode = TreeNode<U>;

    fn map<F>(&self, mapper_fn: F) -> Self::OutputNode
    where
        F: Fn(&T) -> U,
    {
        fn map_tree_node<T, U, F>(node: &TreeNode<T>, mapper_fn: &F) -> TreeNode<U>
        where
            F: Fn(&T) -> U,
        {
            let mapped_node = mapper_fn(node.value());

            if let Some(children) = node.children() {
                TreeNode::from((
                    mapped_node,
                    children
                        .iter()
                        .map(|child| map_tree_node(child, mapper_fn))
                        .collect::<Vec<TreeNode<U>>>(),
                ))
            } else {
                TreeNode::new(mapped_node)
            }
        }

        map_tree_node(self, &mapper_fn)
    }
}

pub trait TopologicalStructure<S = Self> {
    type Link;
    type Value;
    type Node: Node<Value = Self::Value>;

    fn descendant(&self) -> Option<&[Self::Link]>;
    fn ancestors(&self) -> Option<&[Self::Link]>;
}

impl<T> TopologicalStructure for TreeNode<T> {
    type Link = TreeNode<T>;
    type Value = T;
    type Node = TreeNode<T>;

    fn descendant(&self) -> Option<&[Self::Link]> {
        self.children()
    }

    fn ancestors(&self) -> Option<&[Self::Link]> {
        None
    }
}
// fn map<U, F>(&self, mapper_fn: F) -> impl TopologicalStructure<Value = U>
// where
//     F: Fn(&Self::Value) -> U,
// {
//     fn map_tree_node<TT, UU, FF>(node: &TreeNode<TT>, mapper_fn: &FF) -> TreeNode<UU>
//     where
//         FF: Fn(&TT) -> UU,
//     {
//         let mapped_value = mapper_fn(node.value());
//         if let Some(children) = node.children() {
//             TreeNode::from((
//                 mapped_value,
//                 children
//                     .iter()
//                     .map(|child| map_tree_node(child, mapper_fn))
//                     .collect::<Vec<TreeNode<UU>>>(),
//             ))
//         } else {
//             TreeNode::new(mapped_value)
//         }
//     }

//     map_tree_node(self, &mapper_fn)
// }
