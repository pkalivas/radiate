use super::Tree;
use crate::{Eval, TreeNode, node::Node};

/// Implements the [Eval] trait for `Vec<Tree<T>>`. This is a wrapper around a `Vec<Tree<T>>`
/// and allows for the evaluation of each [Tree] in the `Vec` with a single input.
/// This is useful for things like `Ensemble` models where multiple models are used to make a prediction.
///
/// This is a simple implementation that just maps over the `Vec` and calls [Eval] on each [Tree].
impl<T, V> Eval<[V], Vec<V>> for Vec<Tree<T>>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval(&self, inputs: &[V]) -> Vec<V> {
        self.iter().map(|tree| tree.eval(inputs)).collect()
    }
}

/// Implements the [Eval] trait for `Vec<&TreeNode<T>>`. This is a wrapper around a `Vec<&TreeNode<T>>`
/// and allows for the evaluation of each [TreeNode] in the `Vec` with a single input.
/// The len of the input slice must equal the number of nodes in the `Vec`.
impl<T, V> Eval<[V], Vec<V>> for Vec<&TreeNode<T>>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval(&self, inputs: &[V]) -> Vec<V> {
        self.iter().map(|node| node.eval(inputs)).collect()
    }
}

/// Implements the [Eval] trait for [Tree<T>] where `T` is `Eval<[V], V>`. All this really does is
/// call the `eval` method on the root node of the [Tree]. The real work is
/// done in the [TreeNode] implementation below.
impl<T, V> Eval<[V], V> for Tree<T>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval(&self, input: &[V]) -> V {
        self.root()
            .map(|root| root.eval(input))
            .unwrap_or_else(|| panic!("Tree has no root node."))
    }
}

/// Implements the [Eval] trait for `TreeNode<T>` where `T` is `Eval<[V], V>`. This is where the real work is done.
/// It recursively evaluates the [TreeNode] and its children until it reaches a leaf node,
/// at which point it applies the `T`'s eval fn to the input.
///
/// Because a [Tree] has only a single root node, this can only be used to return a single value.
/// We assume here that each leaf can eval the incoming input - this is a safe and the
/// only real logical assumption we can make.
impl<T, V> Eval<[V], V> for TreeNode<T>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval(&self, input: &[V]) -> V {
        if let Some(children) = self.children() {
            let mut inputs = Vec::with_capacity(children.len());

            for child in children {
                inputs.push(child.eval(input));
            }

            return self.value().eval(&inputs);
        }

        self.value().eval(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Op, TreeNode};

    #[test]
    fn test_tree_reduce_simple() {
        let mut root = TreeNode::new(Op::add());

        root.add_child(TreeNode::new(Op::constant(1.0)));
        root.add_child(TreeNode::new(Op::constant(2.0)));

        let result = root.eval(&[]);

        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_tree_reduce_complex() {
        let node = TreeNode::new(Op::add())
            .attach(
                TreeNode::new(Op::mul())
                    .attach(TreeNode::new(Op::constant(2.0)))
                    .attach(TreeNode::new(Op::constant(3.0))),
            )
            .attach(
                TreeNode::new(Op::add())
                    .attach(TreeNode::new(Op::constant(2.0)))
                    .attach(TreeNode::new(Op::var(0))),
            );

        let nine = node.eval(&[1_f32]);
        let ten = node.eval(&[2_f32]);
        let eleven = node.eval(&[3_f32]);

        assert_eq!(nine, 9.0);
        assert_eq!(ten, 10.0);
        assert_eq!(eleven, 11.0);
    }
}
