use super::TreeIterator;
use crate::{Arity, Factory, NodeStore, NodeType, Tree, node::Node};
use radiate_core::genome::{Gene, Valid};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A node in a tree structure that represents a single element with optional children.
///
/// The [TreeNode] struct is a fundamental building block for tree-based genetic programming in Radiate.
/// It represents a node in a tree that can have zero or more child nodes, forming a hierarchical structure.
/// Each node has a value of type T and maintains an optional list of child nodes.
///
/// # Type Parameters
/// * `T` - The type of value stored in the node. This type must implement `Clone`, `PartialEq`, and other traits
///         required by the genetic programming operations.
///
/// # Fields
/// * `value` - The actual value stored in the node
/// * `arity` - Optional Arity that specifies how many children the node can have
/// * `children` - Optional vector of child nodes
///
/// # Examples
/// ```
/// use radiate_gp::{collections::{TreeNode}, Arity, Node};
///
/// // Create a new node with value 42
/// let node = TreeNode::new(42);
///
/// // Create a node with specific arity
/// let node_with_arity = TreeNode::with_arity(42, Arity::Exact(2));
/// let other_node_with_arity = TreeNode::from((42, Arity::Exact(2)));
///
/// assert_eq!(node_with_arity.arity(), other_node_with_arity.arity());
///
/// // Create a node with children
/// let node_with_children = TreeNode::with_children(42, vec![
///     TreeNode::new(1),
///     TreeNode::new(2)
/// ]);
/// let other_node_with_children = TreeNode::from((42, vec![
///     TreeNode::new(1),
///     TreeNode::new(2),
/// ]));
/// ```
///
/// # Node Types and [Arity]
/// The node's type and arity determine its behavior and validity:
/// * `Leaf` nodes have no children (arity is [Arity::Zero])
/// * `Vertex` nodes can have any number of children (arity is [Arity::Any])
/// * `Root` nodes are the starting point of the tree and can have any number of children
///
/// # Tree Operations
/// The struct provides several methods for tree manipulation:
/// * `new()` - Creates a new node with no children
/// * `with_arity()` - Creates a node with a specific arity
/// * `with_children()` - Creates a node with a list of children
/// * `add_child()` - Adds a child to the node
/// * `attach()` - Attaches a child and returns self for method chaining
/// * `detach()` - Removes a child at a specific index
/// * `swap_subtrees()` - Swaps subtrees between two nodes
///
/// # Tree Traversal
/// The struct implements the [TreeIterator] trait, providing three traversal methods:
/// * `iter_pre_order()` - Traverses the tree in pre-order (root, then children)
/// * `iter_post_order()` - Traverses the tree in post-order (children, then root)
/// * `iter_breadth_first()` - Traverses the tree level by level
///
/// # Tree Properties
/// The struct provides methods to query tree properties:
/// * `is_leaf()` - Checks if the node has no children - must have [Arity::Zero]
/// * `size()` - Returns the total number of nodes in the subtree
/// * `height()` - Returns the height of the subtree
///
/// # Validity
/// A node is considered valid based on its arity:
/// * Nodes with [Arity::Zero] must have no children
/// * Nodes with [variant@Arity::Exact] must have exactly n children
/// * Nodes with [Arity::Any] can have any number of children
///
/// # Implementation Details
/// The struct implements several traits:
/// * `Node` - Provides common node behavior and access to value and type information
/// * `Gene` - Enables genetic operations for the node making it compatible with genetic algorithms
/// * `Valid` - Defines validity rules for the node
/// * `Debug` - Provides debug formatting
/// * `Clone`, `PartialEq` - Required for genetic programming operations
/// * `Format` - Provides pretty-printing of the tree structure
///
/// # Evaluation
/// When `T` implements the `Eval` trait, the node can be evaluated with input data:
/// ```rust
/// use radiate_gp::{Op, Eval, TreeNode};
///
/// let tree = TreeNode::new(Op::add())
///     .attach(TreeNode::new(Op::constant(2.0)))
///     .attach(TreeNode::new(Op::constant(3.0)));
///
/// let result = tree.eval(&[]); // Evaluates to 5.0
/// ```
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TreeNode<T> {
    value: T,
    arity: Option<Arity>,
    children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    pub fn new(val: T) -> Self {
        TreeNode {
            value: val,
            arity: None,
            children: None,
        }
    }

    pub fn with_arity(val: T, arity: Arity) -> Self {
        TreeNode {
            value: val,
            arity: Some(arity),
            children: None,
        }
    }

    pub fn with_children<N>(val: T, children: Vec<N>) -> Self
    where
        N: Into<TreeNode<T>>,
    {
        TreeNode {
            value: val,
            arity: None,
            children: Some(children.into_iter().map(|n| n.into()).collect()),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn add_child(&mut self, child: impl Into<TreeNode<T>>) {
        let node = child.into();
        if let Some(children) = self.children.as_mut() {
            children.push(node);
        } else {
            self.children = Some(vec![node]);
        }
    }

    pub fn attach(mut self, other: impl Into<TreeNode<T>>) -> Self {
        self.add_child(other);
        self
    }

    pub fn detach(&mut self, index: usize) -> Option<TreeNode<T>> {
        if let Some(children) = self.children.as_mut() {
            if index < children.len() {
                return Some(children.remove(index));
            }
        }

        None
    }

    pub fn children(&self) -> Option<&[TreeNode<T>]> {
        self.children.as_ref().map(|children| children.as_slice())
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<TreeNode<T>>> {
        self.children.as_mut()
    }

    pub fn take_children(&mut self) -> Option<Vec<TreeNode<T>>> {
        self.children.take()
    }

    #[inline]
    pub fn size(&self) -> usize {
        if let Some(children) = self.children.as_ref() {
            children.iter().fold(1, |acc, child| acc + child.size())
        } else {
            1
        }
    }

    #[inline]
    pub fn height(&self) -> usize {
        if let Some(children) = self.children.as_ref() {
            1 + children
                .iter()
                .map(|child| child.height())
                .max()
                .unwrap_or(0)
        } else {
            0
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut TreeNode<T>> {
        let mut cur = 0;
        Self::get_mut_preorder(self, index, &mut cur)
    }

    #[inline]
    fn get_mut_preorder<'a>(
        node: &'a mut TreeNode<T>,
        target: usize,
        cur: &mut usize,
    ) -> Option<&'a mut TreeNode<T>> {
        if *cur == target {
            return Some(node);
        }

        if let Some(children) = node.children_mut() {
            for child in children {
                *cur += 1;
                if let Some(found) = Self::get_mut_preorder(child, target, cur) {
                    return Some(found);
                }
            }
        }

        None
    }
}

impl<T> Node for TreeNode<T> {
    type Value = T;

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }

    fn node_type(&self) -> NodeType {
        if self.children.is_some() {
            NodeType::Vertex
        } else {
            NodeType::Leaf
        }
    }

    fn arity(&self) -> Arity {
        if let Some(arity) = self.arity {
            arity
        } else if let Some(children) = self.children.as_ref() {
            Arity::Exact(children.len())
        } else {
            match self.node_type() {
                NodeType::Leaf => Arity::Zero,
                NodeType::Vertex => Arity::Any,
                NodeType::Root => Arity::Any,
                _ => Arity::Zero,
            }
        }
    }
}

impl<T> Gene for TreeNode<T>
where
    T: Clone + PartialEq,
{
    type Allele = T;

    fn allele(&self) -> &Self::Allele {
        &self.value
    }

    fn allele_mut(&mut self) -> &mut Self::Allele {
        &mut self.value
    }

    fn new_instance(&self) -> Self {
        TreeNode {
            value: self.value.clone(),
            arity: self.arity,
            children: self.children.as_ref().map(|children| {
                children
                    .iter()
                    .map(|child| child.new_instance())
                    .collect::<Vec<TreeNode<T>>>()
            }),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        TreeNode {
            value: allele.clone(),
            arity: self.arity,
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<T> Valid for TreeNode<T> {
    fn is_valid(&self) -> bool {
        for node in self.iter_breadth_first() {
            match node.arity() {
                Arity::Zero => return node.children.is_none(),
                Arity::Exact(n) => {
                    if node.children.is_none() || node.children.as_ref().unwrap().len() != n {
                        return false;
                    }
                }
                Arity::Any => {}
            }
        }

        true
    }
}

impl<T> Factory<(usize, Option<NodeStore<T>>), Option<TreeNode<T>>> for TreeNode<T>
where
    T: Clone + Default,
{
    fn new_instance(&self, (index, store): (usize, Option<NodeStore<T>>)) -> Option<TreeNode<T>> {
        store
            .map(|store| Tree::with_depth(index, store).take_root())
            .flatten()
    }
}

impl<T: Clone> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        TreeNode {
            value: self.value.clone(),
            arity: self.arity,
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<T: Default> Default for TreeNode<T> {
    fn default() -> Self {
        TreeNode {
            value: T::default(),
            arity: None,
            children: None,
        }
    }
}

impl<T: Debug> Debug for TreeNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:>10?} :: {:<10} {:<12} C: {:?}",
            format!("{:?}", self.node_type())[..3].to_owned(),
            self.arity(),
            format!("{:?}", self.value).to_owned(),
            match &self.children {
                Some(children) => children.len(),
                None => 0,
            },
        )
    }
}

impl<T> From<(T, Arity)> for TreeNode<T> {
    fn from(value: (T, Arity)) -> Self {
        TreeNode::with_arity(value.0, value.1)
    }
}

impl<T> From<(T, Vec<TreeNode<T>>)> for TreeNode<T> {
    fn from(value: (T, Vec<TreeNode<T>>)) -> Self {
        TreeNode::with_children(value.0, value.1)
    }
}

macro_rules! impl_from {
    ($($t:ty),+) => {
        $(
            impl From<$t> for TreeNode<$t> {
                fn from(value: $t) -> Self {
                    TreeNode::new(value)
                }
            }
        )+
    };
}

impl_from!(
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    String,
    bool,
    char,
    usize,
    isize,
    &'static str,
    ()
);

impl<T> From<TreeNode<T>> for Vec<TreeNode<T>> {
    fn from(node: TreeNode<T>) -> Self {
        vec![node]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arity, Eval, NodeType, Op};

    #[test]
    fn test_node_creation() {
        let node = TreeNode::new(42);

        assert_eq!(node.value(), &42);
        assert!(node.is_leaf());
        assert_eq!(node.arity(), Arity::Zero);
        assert_eq!(node.node_type(), NodeType::Leaf);

        let node = TreeNode::with_arity(42, Arity::Exact(2));

        assert_eq!(node.value(), &42);
        assert!(node.is_leaf());
        assert_eq!(node.arity(), Arity::Exact(2));
        assert_eq!(node.node_type(), NodeType::Leaf);

        let node = TreeNode::with_children(42, vec![TreeNode::new(1), TreeNode::new(2)]);

        assert_eq!(node.value(), &42);
        assert!(!node.is_leaf());
        assert_eq!(node.arity(), Arity::Exact(2));
        assert_eq!(node.node_type(), NodeType::Vertex);
    }

    #[test]
    fn test_node_manipulation() {
        let mut node = TreeNode::new(42);
        node.add_child(TreeNode::new(1));

        assert!(!node.is_leaf());
        assert_eq!(node.children().unwrap().len(), 1);
        assert_eq!(node.children().unwrap()[0].value(), &1);

        let node = TreeNode::new(42)
            .attach(TreeNode::new(1))
            .attach(TreeNode::new(2));

        assert_eq!(node.children().unwrap().len(), 2);
        assert_eq!(node.children().unwrap()[0].value(), &1);
        assert_eq!(node.children().unwrap()[1].value(), &2);

        let mut node = TreeNode::with_children(42, vec![TreeNode::new(1), TreeNode::new(2)]);
        let detached = node.detach(0);

        assert!(detached.is_some());
        assert_eq!(detached.unwrap().value(), &1);
        assert_eq!(node.children().unwrap().len(), 1);
        assert_eq!(node.children().unwrap()[0].value(), &2);

        assert!(node.detach(5).is_none());
    }

    #[test]
    fn test_tree_properties() {
        let node = TreeNode::new(42).attach(TreeNode::new(1)).attach(
            TreeNode::new(2)
                .attach(TreeNode::new(3))
                .attach(TreeNode::new(4)),
        );

        assert_eq!(node.size(), 5);
        assert_eq!(node.height(), 2);

        let leaf = TreeNode::new(42);

        assert_eq!(leaf.size(), 1);
        assert_eq!(leaf.height(), 0);
    }

    #[test]
    fn test_tree_traversal() {
        // Create a tree:
        //       42
        //      /  \
        //     1    2
        //         / \
        //        3   4
        let node = TreeNode::new(42).attach(TreeNode::new(1)).attach(
            TreeNode::new(2)
                .attach(TreeNode::new(3))
                .attach(TreeNode::new(4)),
        );

        let pre_order: Vec<i32> = node.iter_pre_order().map(|n| *n.value()).collect();
        assert_eq!(pre_order, vec![42, 1, 2, 3, 4]);

        let post_order: Vec<i32> = node.iter_post_order().map(|n| *n.value()).collect();
        assert_eq!(post_order, vec![1, 3, 4, 2, 42]);

        let bfs: Vec<i32> = node.iter_breadth_first().map(|n| *n.value()).collect();
        assert_eq!(bfs, vec![42, 1, 2, 3, 4]);
    }

    #[test]
    fn test_node_validity() {
        let node = TreeNode::with_arity(42, Arity::Zero);
        assert!(node.is_valid());

        let mut node = TreeNode::with_arity(42, Arity::Zero);
        node.add_child(TreeNode::new(1));
        assert!(!node.is_valid());

        let node = TreeNode::with_arity(42, Arity::Exact(2))
            .attach(TreeNode::new(1))
            .attach(TreeNode::new(2));
        assert!(node.is_valid());

        let mut node = TreeNode::with_arity(42, Arity::Exact(2));
        node.add_child(TreeNode::new(1));
        assert!(!node.is_valid());

        let node = TreeNode::with_arity(42, Arity::Any)
            .attach(TreeNode::new(1))
            .attach(TreeNode::new(2))
            .attach(TreeNode::new(3));
        assert!(node.is_valid());
    }

    #[test]
    fn test_node_evaluation() {
        // Test simple arithmetic expression: (1 + 2) * 3
        let node = TreeNode::new(Op::mul())
            .attach(
                TreeNode::new(Op::add())
                    .attach(TreeNode::new(Op::constant(1.0)))
                    .attach(TreeNode::new(Op::constant(2.0))),
            )
            .attach(TreeNode::new(Op::constant(3.0)));

        let result = node.eval(&[]);
        assert_eq!(result, 9.0);

        // Test expression with variables: (x + 2) * 3
        let node = TreeNode::new(Op::mul())
            .attach(
                TreeNode::new(Op::add())
                    .attach(TreeNode::new(Op::var(0)))
                    .attach(TreeNode::new(Op::constant(2.0))),
            )
            .attach(TreeNode::new(Op::constant(3.0)));

        assert_eq!(node.eval(&[1.0]), 9.0);
        assert_eq!(node.eval(&[2.0]), 12.0);
        assert_eq!(node.eval(&[3.0]), 15.0);
    }

    #[test]
    fn test_cloning_and_equality() {
        let node1 = TreeNode::new(42)
            .attach(TreeNode::new(1))
            .attach(TreeNode::new(2));

        let node2 = node1.clone();
        assert_eq!(node1, node2);

        let node3 = TreeNode::new(43)
            .attach(TreeNode::new(1))
            .attach(TreeNode::new(2));
        assert_ne!(node1, node3);

        let node4 = TreeNode::new(42).attach(TreeNode::new(1));
        assert_ne!(node1, node4);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_node_can_serialize() {
        let root = TreeNode::new(42)
            .attach(TreeNode::new(1))
            .attach(
                TreeNode::new(2)
                    .attach(TreeNode::new(3))
                    .attach(TreeNode::new(4)),
            )
            .attach(TreeNode::with_arity(3, Arity::Exact(2)));

        let serialized = serde_json::to_string(&root).unwrap();
        let deserialized: TreeNode<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(root, deserialized);
    }
}
