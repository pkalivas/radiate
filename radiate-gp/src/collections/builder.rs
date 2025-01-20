/// A trait for types that can be built into a final value.
///
/// This is a quality of life trait to help with the builder pattern or
/// with things that can be 'built'. There are a multitude of things in
/// the `radiate_gp` crate that rely on being 'built', and this trait is
/// meant to abstract that out. For example, the `Tree`, `Graph', `TreeChromosome`,
/// `GraphChromosome` can all be 'built' and thus implement this trait.
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
pub trait Builder {
    type Output;
    fn build(&self) -> Self::Output;
}
