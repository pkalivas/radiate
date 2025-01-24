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
/// use radiate_gp::{Op, Reduce, TreeNode};
///
/// let mut other_root = TreeNode::new(Op::add())
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
/// let result = root.reduce(&vec![1_f32]);
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
pub trait Reduce<T> {
    type Input;
    type Output;

    fn reduce(&mut self, input: &Self::Input) -> Self::Output;
}
