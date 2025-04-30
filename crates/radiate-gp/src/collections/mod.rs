pub mod factory;
pub mod format;
pub mod graphs;
pub mod node;
pub mod store;
pub mod trees;

pub use factory::*;
pub use format::*;
pub use graphs::{
    Direction, Graph, GraphAggregate, GraphChromosome, GraphCodex, GraphCrossover, GraphEvaluator,
    GraphIterator, GraphMutator, GraphNode, GraphReplacement, NeatDistance,
};
pub use node::{Node, NodeType};
pub use store::{NodeStore, NodeValue};
pub use trees::{
    HoistMutator, Tree, TreeChromosome, TreeCodex, TreeCrossover, TreeIterator, TreeNode,
};
/// [Eval] trait is used to evaluate a [Tree] or [Graph] of node's.
/// It is implemented directly on the [Tree] and [TreeNode] types as well as
/// on the [GraphEvaluator] struct.
///
/// The [Eval] trait and subsequent method is used to transform the `Input` into
/// the `Output`. This is extremely useful for evaluating the [Graph] or [Tree] with a given input
/// as traversing each can be very slow or sometimes cumbersome to do manually.
///
/// # Example
/// ```rust
/// use radiate_gp::{Op, Eval, TreeNode};
///
/// let root = TreeNode::new(Op::add())
///     .attach(
///         TreeNode::new(Op::mul())
///             .attach(TreeNode::new(Op::constant(2.0)))
///             .attach(TreeNode::new(Op::constant(3.0))),
///     )
///     .attach(
///         TreeNode::new(Op::add())
///             .attach(TreeNode::new(Op::constant(2.0)))
///             .attach(TreeNode::new(Op::var(0))),
///     );
///
/// // And the result of evaluating this tree with an input of `1` would be:
/// let result = root.eval(&vec![1_f32]);
/// assert_eq!(result, 9.0);
/// ```
/// This creates a `Tree` that looks like:
/// ```text
///      +
///    /   \
///   *     +
///  / \   / \
/// 2  3  2   x
/// ```
/// Where `x` is the first variable in the input.
///
/// This can also be thought of (and is functionally equivalent) as:
/// ```text
/// f(x) = (2 * 3) + (2 + x)
/// ```
pub trait Eval<I: ?Sized, O> {
    fn eval(&self, input: &I) -> O;
}

pub trait EvalMut<I: ?Sized, O> {
    fn eval_mut(&mut self, input: &I) -> O;
}

impl<I: ?Sized, O> EvalMut<I, O> for dyn Eval<I, O> {
    fn eval_mut(&mut self, input: &I) -> O {
        self.eval(input)
    }
}
