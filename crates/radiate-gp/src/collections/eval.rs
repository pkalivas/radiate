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

impl<I: ?Sized, O, T> EvalMut<I, O> for T
where
    T: Eval<I, O>,
{
    #[inline]
    fn eval_mut(&mut self, input: &I) -> O {
        self.eval(input)
    }
}

pub trait EvalInto<I: ?Sized, O: ?Sized> {
    fn eval_into(&self, input: &I, buffer: &mut O);
}

pub trait EvalIntoMut<I: ?Sized, O: ?Sized> {
    fn eval_into_mut(&mut self, input: &I, buffer: &mut O);
}

impl<I: ?Sized, O: ?Sized, T> EvalIntoMut<I, O> for T
where
    T: EvalInto<I, O>,
{
    #[inline]
    fn eval_into_mut(&mut self, input: &I, buffer: &mut O) {
        self.eval_into(input, buffer)
    }
}

/// [EvalInto] implementation for closures that take an input and a mutable output buffer.
impl<F, I: ?Sized, O: ?Sized> EvalInto<I, O> for F
where
    F: Fn(&I, &mut O),
{
    #[inline]
    fn eval_into(&self, input: &I, buffer: &mut O) {
        (self)(input, buffer)
    }
}

/// [Eval] implementation for closures that take an input and return an output.
impl<F, I: ?Sized, O> Eval<I, O> for F
where
    F: Fn(&I) -> O,
{
    #[inline]
    fn eval(&self, input: &I) -> O {
        (self)(input)
    }
}
