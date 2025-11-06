use crate::{TreeIterator, collections::TreeNode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
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
/// assert_eq!(result, 9.0);
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    use crate::{Arity, Node, NodeType, Op, TreeCrossover, TreeIterator};

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

        let copy_one = tree_one.clone();
        let copy_two = tree_two.clone();

        TreeCrossover::cross_nodes(tree_one.as_mut(), tree_two.as_mut(), usize::MAX);

        let new_one = tree_one.clone();
        let new_two = tree_two.clone();

        // Ensure that subtrees have been swapped
        assert_ne!(copy_one, new_one);
        assert_ne!(copy_two, new_two);
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

    #[test]
    fn test_tree_with_mixed_arity() {
        let store = vec![
            (
                NodeType::Vertex,
                vec![
                    Op::add(),         // Binary operator
                    Op::constant(1.0), // Constant
                    Op::sigmoid(),     // Unary operator - gets treated as arity 2
                ],
            ),
            (NodeType::Leaf, vec![Op::constant(2.0)]),
        ];
        let tree = Tree::with_depth(3, store);

        // Verify that nodes have appropriate arity based on their type
        for node in tree.iter_breadth_first() {
            match node.value() {
                Op::Fn(name, arity, _) if *name == "add" || *name == "sub" || *name == "mul" => {
                    assert_eq!(**arity, 2, "Binary operator should have arity 2")
                }
                Op::Const(_, _) => assert_eq!(*node.arity(), 0, "Constant should have arity 0"),
                Op::Fn(name, arity, _) if *name == "sigmoid" => {
                    assert!(
                        vec![0, 1, 2].contains(&**arity),
                        "Unary operator should have arity 0 or 1 or 2"
                    )
                }
                _ => (), // Other ops can be ignored for this test
            }
        }
    }

    #[test]
    fn test_tree_with_zero_arity() {
        let store = vec![
            (NodeType::Vertex, vec![Op::constant(1.0)]), // Constants have zero arity
            (NodeType::Leaf, vec![Op::constant(2.0)]),
        ];
        let tree = Tree::with_depth(2, store);

        // Verify that each vertex node has no children
        for node in tree.iter_breadth_first() {
            println!("Node: {:?}", node);

            assert_eq!(*node.arity(), 0, "Vertex node should have zero arity");
            assert!(
                node.children().is_none(),
                "Vertex node should have no children"
            );
        }
    }

    #[test]
    fn test_tree_with_exact_arity() {
        let store = vec![
            (NodeType::Vertex, vec![Op::add(), Op::sub()]), // Binary operators
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let tree = Tree::with_depth(2, store);

        // Verify that each vertex node has exactly 2 children
        for node in tree.iter_breadth_first() {
            if node.node_type() == NodeType::Vertex {
                assert_eq!(node.arity(), Arity::Exact(2));
                assert_eq!(node.children().unwrap().len(), 2);
            }
        }
    }

    #[test]
    fn test_tree_with_only_leaf_nodes() {
        let store = vec![(NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)])];
        let tree = Tree::with_depth(3, store);
        assert!(tree.root().is_some());
        assert_eq!(tree.root().unwrap().node_type(), NodeType::Leaf);
        assert_eq!(tree.size(), 1);
        assert_eq!(tree.height(), 0);
    }

    #[test]
    fn test_tree_with_empty_store() {
        let empty_store: Vec<(NodeType, Vec<Op<f32>>)> = vec![];
        let tree = Tree::with_depth(3, empty_store);

        // Root will be a default node since no valid nodes were provided
        assert!(tree.root().is_some());
        assert_eq!(tree.size(), 1);
        assert_eq!(tree.height(), 0);
    }

    #[test]
    fn test_tree_debug() {
        let tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::constant(1.0)))
                .attach(TreeNode::new(Op::constant(2.0))),
        );

        let debug_str = format!("{:?}", tree);
        assert!(debug_str.contains("Tree {"));
        assert!(debug_str.contains("add"));
        assert!(debug_str.contains("C"));
    }

    #[test]
    fn test_tree_as_ref_as_mut() {
        let mut tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::constant(1.0)))
                .attach(TreeNode::new(Op::constant(2.0))),
        );

        // Test AsRef
        let root_ref: &TreeNode<Op<f32>> = tree.as_ref();
        assert_eq!(root_ref.value(), &Op::add());
        assert_eq!(root_ref.children().unwrap().len(), 2);

        // Test AsMut
        let root_mut: &mut TreeNode<Op<f32>> = tree.as_mut();
        assert_eq!(root_mut.value(), &Op::add());

        root_mut
            .children_mut()
            .unwrap()
            .push(TreeNode::new(Op::constant(3.0))); // Add a new child
        assert_eq!(root_mut.children().unwrap().len(), 3); // Now should have 3 children
        // assert!(!tree.as_ref().is_valid()); // Invalid since we added a child without updating size
    }

    #[test]
    fn test_tree_root_operations() {
        // Test root operations on empty tree
        let mut empty_tree = Tree::<Op<f32>>::default();
        assert!(empty_tree.root().is_none());
        assert!(empty_tree.root_mut().is_none());
        assert!(empty_tree.take_root().is_none());

        // Test root operations on non-empty tree
        let tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::constant(1.0)))
                .attach(TreeNode::new(Op::constant(2.0))),
        );

        // Test root()
        let root = tree.root().unwrap();
        assert_eq!(root.value(), &Op::add());
        assert_eq!(root.children().unwrap().len(), 2);

        // Test take_root()
        let root = tree.take_root().unwrap();
        assert_eq!(root.value(), &Op::add());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_tree_can_serde() {
        use crate::Eval;

        let store = vec![
            (
                NodeType::Vertex,
                vec![
                    Op::add(),
                    Op::sub(),
                    Op::mul(),
                    Op::div(),
                    Op::sigmoid(),
                    Op::tanh(),
                ],
            ),
            (
                NodeType::Leaf,
                vec![Op::constant(1.0), Op::constant(2.0), Op::var(0)],
            ),
        ];

        let tree = Tree::with_depth(5, store);

        let eval_before = tree.eval(&[3.0]);

        let serialized = serde_json::to_string(&tree).expect("Failed to serialize tree");
        let deserialized: Tree<Op<f32>> =
            serde_json::from_str(&serialized).expect("Failed to deserialize tree");

        let eval_after = deserialized.eval(&[3.0]);

        assert_eq!(
            eval_before, eval_after,
            "Tree evaluation should match before and after serialization"
        );
        assert_eq!(tree, deserialized);
    }
}
