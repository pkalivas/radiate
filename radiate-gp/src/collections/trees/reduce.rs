use crate::{Op, Reduce, TreeNode};

use super::Tree;

/// Implements the `Reduce` trait for `Tree<Op<T>>`. All this really does is
/// call the `reduce` method on the root node of the `Tree`. The real work is
/// done in the `TreeNode` implementation below.
impl<T: Clone> Reduce<T> for Tree<Op<T>> {
    type Input = Vec<T>;
    type Output = T;

    #[inline]
    fn reduce(&mut self, input: &Self::Input) -> Self::Output {
        self.root_mut()
            .map(|root| root.reduce(input))
            .unwrap_or_else(|| panic!("Tree has no root node."))
    }
}

/// Implements the `Reduce` trait for `TreeNode<Op<T>>`. This is where the real work is done.
/// It recursively evaluates the `TreeNode` and its children until it reaches a leaf node,
/// at which point it applies the `Op` to the input.
///
/// Because a `Tree` has only a single root node, this can only be used to return a single value.
/// But, due to the structure and functionality of the `Op<T>`, we can have a multitude of `Inputs`
impl<T: Clone> Reduce<T> for TreeNode<Op<T>> {
    type Input = Vec<T>;
    type Output = T;

    #[inline]
    fn reduce(&mut self, input: &Self::Input) -> Self::Output {
        fn eval<T: Clone>(node: &TreeNode<Op<T>>, curr_input: &Vec<T>) -> T {
            if node.is_leaf() {
                node.value().apply(curr_input)
            } else {
                if let Some(children) = &node.children() {
                    let mut inputs = Vec::with_capacity(children.len());

                    for child in *children {
                        inputs.push(eval(child, curr_input));
                    }

                    return node.value().apply(&inputs);
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

        let result = root.reduce(&vec![]);

        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_tree_reduce_complex() {
        let mut root = TreeNode::new(Op::add())
            .attach(
                TreeNode::new(Op::mul())
                    .attach(TreeNode::new(Op::value(2.0)))
                    .attach(TreeNode::new(Op::value(3.0))),
            )
            .attach(
                TreeNode::new(Op::add())
                    .attach(TreeNode::new(Op::value(2.0)))
                    .attach(TreeNode::new(Op::var(0))),
            );

        let nine = root.reduce(&vec![1_f32]);
        let ten = root.reduce(&vec![2_f32]);
        let eleven = root.reduce(&vec![3_f32]);

        assert_eq!(nine, 9.0);
        assert_eq!(ten, 10.0);
        assert_eq!(eleven, 11.0);
    }
}
