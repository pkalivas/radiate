/// [Eval] trait is used to evaluate a [Tree] or [Graph] of node's.
/// It is implemented directly on the GP structures to allow for easy and dynamic
/// evaluation of the structures with a given input.
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

impl<I, O, T> EvalMut<I, O> for T
where
    I: ?Sized,
    T: Eval<I, O>,
{
    #[inline]
    fn eval_mut(&mut self, input: &I) -> O {
        self.eval(input)
    }
}

/// [EvalInto] trait is used to evaluate anything that implements the trait into a mutable buffer.
/// In some cases where we can cache a buffer to write into, this can avoid allocations and massively
/// improve performance. Just like the [Eval] & [EvalMut] traits the mutable version offers a way for the
/// implementing type to mutate its internal state if needed. These are also implemented for the GP structures.
///
/// # Example
/// ```rust
/// use radiate_gp::*;
///
/// // Create a simple graph that adds two inputs together
/// // This is functionaly equivalent to f(x, y) = x + y
/// let store = node_store! {
///     Input => vec![Op::var(0), Op::var(1)],
///     Output => vec![Op::add()]
/// };
///
/// let mut graph = Graph::directed(2, 1, store);
///
/// // Define a buffer to store the output
/// let mut output_buffer = vec![vec![0.0]];
/// graph.eval_into(&vec![vec![5.0, 10.0]], &mut output_buffer);
///
/// // Check the output - it should be 5 + 10 = 15
/// assert_eq!(output_buffer[0][0], 15.0);
/// ```
pub trait EvalInto<I: ?Sized, O: ?Sized> {
    fn eval_into(&self, input: &I, buffer: &mut O);
}

pub trait EvalIntoMut<I: ?Sized, O: ?Sized> {
    fn eval_into_mut(&mut self, input: &I, buffer: &mut O);
}

impl<I, O, T> EvalIntoMut<I, O> for T
where
    I: ?Sized,
    O: ?Sized,
    T: EvalInto<I, O>,
{
    #[inline]
    fn eval_into_mut(&mut self, input: &I, buffer: &mut O) {
        self.eval_into(input, buffer)
    }
}

/// --- Blanket Implementations ---
///
/// Below are blanket implementations for closures to make it easy to use them
/// wherever an [Eval] or [EvalInto] is required.
impl<F, I, O> EvalInto<I, O> for F
where
    I: ?Sized,
    O: ?Sized,
    F: Fn(&I, &mut O),
{
    #[inline]
    fn eval_into(&self, input: &I, buffer: &mut O) {
        (self)(input, buffer)
    }
}

/// [Eval] implementation for closures that take an input and return an output.
impl<F, I, O> Eval<I, O> for F
where
    I: ?Sized,
    F: Fn(&I) -> O,
{
    #[inline]
    fn eval(&self, input: &I) -> O {
        (self)(input)
    }
}
