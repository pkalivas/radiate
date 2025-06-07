use crate::{TreeIterator, collections::TreeNode};
use std::fmt::Debug;

/// A tree structure that represents a hierarchical collection of nodes.
///
/// The `Tree` struct is a fundamental data structure in Radiate's genetic programming system.
/// It provides a way to represent and manipulate tree-based expressions, where each node
/// can have zero or more child nodes. The tree is rooted, meaning it has a single root node
/// from which all other nodes descend.
///
/// # Type Parameters
/// * `T` - The type of value stored in each node. This type must implement `Clone`, `PartialEq`,
///         and other traits required by the genetic programming operations.
///
/// # Fields
/// * `root` - An optional `TreeNode<T>` that serves as the root of the tree. When `None`,
///            the tree is considered empty.
///
/// # Examples
/// ```
/// use radiate_gp::{Tree, TreeNode, Op, Eval};
///
/// // Create a simple tree representing the expression (1 + 2) * 3
/// let tree = Tree::new(
///     TreeNode::new(Op::mul())
///         .attach(
///             TreeNode::new(Op::add())
///                 .attach(TreeNode::new(Op::constant(1.0)))
///                 .attach(TreeNode::new(Op::constant(2.0)))
///         )
///         .attach(TreeNode::new(Op::constant(3.0)))
/// );
///
/// // Evaluate the tree
/// let result = tree.eval(&[]); // Evaluates to 9.0
/// ```
///
/// # Tree Creation
/// The struct provides several ways to create trees:
/// * `new()` - Creates a tree with a given root node
/// * `with_depth()` - Creates a tree of specified depth using nodes from a `NodeStore`
/// * `default()` - Creates an empty tree
///
/// # Tree Operations
/// The struct provides methods for tree manipulation and traversal:
/// * `root()` - Gets a reference to the root node
/// * `root_mut()` - Gets a mutable reference to the root node
/// * `take_root()` - Takes ownership of the root node
/// * `size()` - Returns the total number of nodes in the tree
/// * `height()` - Returns the height of the tree
///
/// # Tree Traversal
/// The struct implements the `TreeIterator` trait, providing three traversal methods:
/// * `iter_pre_order()` - Traverses the tree in pre-order (root, then children)
/// * `iter_post_order()` - Traverses the tree in post-order (children, then root)
/// * `iter_breadth_first()` - Traverses the tree level by level
///
/// # Tree Building
/// The struct provides a builder pattern for creating trees of specific depths:
/// ```rust
/// use radiate_gp::{Tree, NodeType, Op};
///
/// let store = vec![
///     (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
///     (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
/// ];
///
/// // Create a tree of depth 3
/// let tree = Tree::with_depth(3, store);
/// assert_eq!(tree.height(), 3);
/// ```
///
/// # Tree Evaluation
/// When `T` implements the `Eval` trait, the tree can be evaluated with input data:
/// ```rust
/// use radiate_gp::{Tree, TreeNode, Op, Eval};
///
/// let tree = Tree::new(
///     TreeNode::new(Op::add())
///         .attach(TreeNode::new(Op::var(0)))
///         .attach(TreeNode::new(Op::constant(2.0)))
/// );
///
/// assert_eq!(tree.eval(&[1.0]), 3.0);
/// assert_eq!(tree.eval(&[2.0]), 4.0);
/// ```
///
/// # Tree Properties
/// The tree maintains several important properties:
/// * It is always rooted (has a single root node)
/// * It is acyclic (no node is its own ancestor)
/// * Each node can have zero or more children
/// * The tree's height is the length of the longest path from root to leaf
/// * The tree's size is the total number of nodes
///
/// # Implementation Details
/// The struct implements several traits:
/// * `Clone` - Allows cloning of the entire tree structure
/// * `PartialEq` - Enables equality comparison between trees
/// * `Default` - Provides a way to create an empty tree
/// * `Debug` - Provides debug formatting for the tree
/// * `AsRef<TreeNode<T>>` - Allows treating the tree as a reference to its root node
/// * `AsMut<TreeNode<T>>` - Allows treating the tree as a mutable reference to its root node
///
/// # Genetic Programming
/// The `Tree` struct is particularly useful in genetic programming as it can represent:
/// * Mathematical expressions
/// * Program syntax trees
/// * Decision trees
/// * Other hierarchical structures
#[derive(Clone, PartialEq, Default)]
pub struct Tree<T> {
    root: Option<TreeNode<T>>,
}

impl<T> Tree<T> {
    pub fn new(root: impl Into<TreeNode<T>>) -> Self {
        Tree {
            root: Some(root.into()),
        }
    }

    pub fn root(&self) -> Option<&TreeNode<T>> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut TreeNode<T>> {
        self.root.as_mut()
    }

    pub fn take_root(self) -> Option<TreeNode<T>> {
        self.root
    }

    pub fn size(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.size())
    }

    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.height())
    }
}

impl<T> AsRef<TreeNode<T>> for Tree<T> {
    fn as_ref(&self) -> &TreeNode<T> {
        self.root.as_ref().unwrap()
    }
}

impl<T> AsMut<TreeNode<T>> for Tree<T> {
    fn as_mut(&mut self) -> &mut TreeNode<T> {
        self.root.as_mut().unwrap()
    }
}

impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree {{\n")?;
        for node in self.iter_breadth_first() {
            write!(f, "  {:?}\n", node)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Node, NodeType, Op, TreeIterator};

    #[test]
    fn test_swap_subtrees() {
        let mut tree_one = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::constant(1.0)))
                .attach(TreeNode::new(Op::constant(2.0))),
        );

        let mut tree_two = Tree::new(
            TreeNode::new(Op::mul())
                .attach(TreeNode::new(Op::constant(3.0)))
                .attach(TreeNode::new(Op::constant(4.0))),
        );

        tree_one.as_mut().swap_subtrees(tree_two.as_mut(), 1, 1);

        let values_one = tree_one
            .iter_breadth_first()
            .filter_map(|n| match &n.value() {
                Op::Const(_, v) => Some(*v),
                _ => None,
            })
            .collect::<Vec<f32>>();

        assert_eq!(values_one, vec![3.0, 2.0]);
    }

    #[test]
    fn test_size() {
        let tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::from(Op::constant(1.0)))
                .attach(TreeNode::from(Op::constant(2.0))),
        );

        assert_eq!(tree.size(), 3);
    }

    #[test]
    fn test_depth() {
        let store = vec![
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];

        let tree = Tree::with_depth(5, store);
        assert_eq!(tree.height(), 5);
    }
}
