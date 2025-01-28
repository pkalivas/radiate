pub mod graphs;
pub mod trees;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub use graphs::{
    Direction, Graph, GraphAggregate, GraphChromosome, GraphCodex, GraphCrossover, GraphEvaluator,
    GraphMutator, GraphNode, GraphTopologicalIterator, NodeMutate, NodeType,
};

use radiate::random_provider;
pub use trees::{
    Tree, TreeBuilder, TreeChromosome, TreeCodex, TreeCrossover, TreeIterator, TreeMutator,
    TreeNode,
};

use crate::{ops::Arity, Op};

/// A trait for types that can be built into a final value.
///
/// This is a quality of life trait to help with the builder pattern or
/// with things that can be 'built'. There are a multitude of things in
/// the `radiate_gp` crate that rely on being 'built', and this trait is
/// meant to abstract that out. For example, the `Tree` and `Graph',
/// can all be 'built' and thus implement this trait.
///
/// # Example building a `Tree<Op<f32>>`:
/// ```rust
/// use radiate_gp::{Builder, Tree, Op, TreeBuilder};
///
/// let builder = TreeBuilder::new(3)
///     .with_gates(vec![Op::add(), Op::mul()])
///     .with_leafs(vec![Op::value(1.0), Op::value(2.0)]);
///
/// // build the tree using the 'Builder' trait resulting in
/// // the defined `Output` type of the `Builder` trait which
/// // in this case is a `Tree`.
/// let tree = builder.build();
///
/// assert!(tree.height() == 3);
/// assert!(tree.size() == 15);
/// ```
///
/// The above will result in a tree that looks something like:
/// ```text
///       +
///     /   \
///    +     *
///   / \   / \
///  1   2 1   2
/// ```
/// **Note**: The above is not guaranteed, but is a good example of what
/// the tree will look like. It isn't guaranteed because the `TreeBuilder`
/// uses the `random_provider` to pick the value (`Op<f32>` in this case) for each node.
pub trait Builder {
    type Output;
    fn build(&self) -> Self::Output;
}

/// `Reduce` trait is used to evaluate a `Tree` or `Graph` of `Node`s.
/// It is implemented directly on the `Tree` and `TreeNode` types as well as
/// on the `GraphReducer` struct.
///
/// The `reduce` trait and subsequent method is used to transform the `Input` into
/// the `Output`. This is extremely useful for evaluating the `Graph` or `Tree` with a given input
/// as traversing each can be very slow or sometimes cumbersome to do manually.
///
/// # Example
/// ```rust
/// use radiate_gp::{Op, Eval, TreeNode};
///
/// let root = TreeNode::new(Op::add())
///     .attach(
///         TreeNode::new(Op::mul())
///             .attach(TreeNode::new(Op::value(2.0)))
///             .attach(TreeNode::new(Op::value(3.0))),
///     )
///     .attach(
///         TreeNode::new(Op::add())
///             .attach(TreeNode::new(Op::value(2.0)))
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

/// A trait for types that can be created from a given input.
///
/// TODO: Document this trait.
pub trait Factory<T> {
    type Input;
    fn new_instance(&self, input: Self::Input) -> T;
}

pub trait Generator {
    type Input;
    type Output;

    fn generate(&self, input: Self::Input) -> Self::Output;
}

pub trait Store<K, V> {
    fn map<F, R>(&self, key: K, f: F) -> Option<R>
    where
        F: Fn(&V) -> R;

    fn get_or_insert_with<F>(&self, key: K, f: F) -> V
    where
        V: Clone,
        F: Fn() -> V;
}

pub trait Repair {
    fn try_repair(&mut self) -> bool;
}
