use crate::{Eval, TreeNode};

use super::Tree;

/// Implements the `Reduce` trait for `Tree<Op<T>>`. All this really does is
/// call the `reduce` method on the root node of the `Tree`. The real work is
/// done in the `TreeNode` implementation below.
impl<T: Clone> Eval<[T], T> for Tree<T> {
    #[inline]
    fn eval(&self, input: &[T]) -> T {
        self.root()
            .map(|root| root.eval(input))
            .unwrap_or_else(|| panic!("Tree has no root node."))
    }
}

/// Implements the `Reduce` trait for `TreeNode<Op<T>>`. This is where the real work is done.
/// It recursively evaluates the `TreeNode` and its children until it reaches a leaf node,
/// at which point it applies the `Op` to the input.
///
/// Because a `Tree` has only a single root node, this can only be used to return a single value.
/// But, due to the structure and functionality of the `Op<T>`, we can have a multitude of `Inputs`
impl<T> Eval<[T], T> for TreeNode<T>
where
    T: Clone,
{
    #[inline]
    fn eval(&self, input: &[T]) -> T {
        fn eval<T>(node: &TreeNode<T>, curr_input: &[T]) -> T
        where
            T: Clone,
        {
            if node.is_leaf() {
                node.value().eval(curr_input)
            } else {
                if let Some(children) = node.children() {
                    let mut inputs = Vec::with_capacity(children.len());

                    for child in children {
                        inputs.push(eval(&child, curr_input));
                    }

                    return node.value().eval(&inputs);
                }

                panic!("Node is not a leaf and has no children - this should never happen.");
            }
        }

        eval(self, input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Op, TreeNode};

    use super::*;

    #[test]
    fn test_tree_reduce_simple() {
        let mut root = TreeNode::new(Op::add());

        root.add_child(TreeNode::new(Op::value(1.0)));
        root.add_child(TreeNode::new(Op::value(2.0)));

        let result = root.eval(&vec![]);

        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_tree_reduce_complex() {
        let tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(
                    TreeNode::new(Op::mul())
                        .attach(TreeNode::new(Op::value(2.0)))
                        .attach(TreeNode::new(Op::value(3.0))),
                )
                .attach(
                    TreeNode::new(Op::add())
                        .attach(TreeNode::new(Op::value(2.0)))
                        .attach(TreeNode::new(Op::var(0))),
                ),
        );

        let nine = tree.eval(&vec![1_f32]);
        let ten = tree.eval(&vec![2_f32]);
        let eleven = tree.eval(&vec![3_f32]);

        assert_eq!(nine, 9.0);
        assert_eq!(ten, 10.0);
        assert_eq!(eleven, 11.0);
    }
}
