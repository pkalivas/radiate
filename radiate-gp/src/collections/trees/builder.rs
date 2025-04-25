use crate::collections::{Tree, TreeNode};
use crate::node::Node;
use crate::{Arity, Factory, NodeStore, NodeType};

const NUM_CHILDREN_ANY: usize = 2;

impl<T: Clone + Default> Tree<T> {
    /// Create a tree with the given depth, where each node is a random node from the node store.
    /// This obeys the rules of the [NodeStore]'s [NodeType]'s arity, and will create a tree
    /// that is as balanced as possible.
    ///
    /// Note that the root node will try to be a [NodeType::Root] if it is available in the
    /// [NodeStore], otherwise it will be a [NodeType::Vertex]. This allows caller's to specify what
    /// the root node is if desired, otherwise it will be a random vertex node from the [NodeStore].
    ///
    /// # The [NodeStore] must contain at least one [NodeType::Root] or one [NodeType::Vertex]
    ///
    /// # Arguments
    /// * `depth` - The depth of the tree.
    /// * `nodes` - The node store to use for the tree.
    ///
    /// # Returns
    /// A tree with the given depth, where each node is a random node from the node store.
    pub fn with_depth(depth: usize, nodes: impl Into<NodeStore<T>>) -> Self {
        let store = nodes.into();

        let mut root = if store.contains_type(NodeType::Root) {
            store.new_instance(NodeType::Root)
        } else {
            store.new_instance(NodeType::Vertex)
        };

        if root.arity() == Arity::Any {
            for _ in 0..NUM_CHILDREN_ANY {
                root.add_child(Self::grow(depth - 1, &store));
            }
        } else {
            for _ in 0..*root.arity() {
                root.add_child(Self::grow(depth - 1, &store));
            }
        }

        Tree::new(root)
    }

    /// Recursively grow a tree from the given depth, where each node is a random node from the
    /// node store. If the depth is 0, then a leaf node is returned. Otherwise, a vertex node is
    /// returned with children that are grown from the given depth.
    /// This obeys the rules of the [NodeStore]'s [NodeType]'s arity, and will create a tree
    /// that is as balanced as possible.
    ///
    /// # Arguments
    /// * `current_depth` - The current depth of the tree.
    /// * `store` - The node store to use for the tree.
    ///
    /// # Returns
    /// A tree node with the given depth, where each node is a random node from the node store.
    fn grow(current_depth: usize, store: &NodeStore<T>) -> TreeNode<T> {
        if current_depth == 0 {
            return store.new_instance(NodeType::Leaf);
        }

        let mut parent = store.new_instance(NodeType::Vertex);
        let num_children = match parent.arity() {
            Arity::Zero => 0,
            Arity::Exact(n) => n,
            Arity::Any => NUM_CHILDREN_ANY,
        };

        for _ in 0..num_children {
            parent.add_child(Self::grow(current_depth - 1, store));
        }

        parent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Op, TreeIterator};

    #[test]
    fn test_tree_builder_depth_two() {
        let store = vec![
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let tree = Tree::with_depth(2, store);

        assert!(tree.root().is_some());
        assert_eq!(tree.root().unwrap().children().unwrap().len(), 2);
        assert_eq!(tree.height(), 2);
        assert_eq!(tree.size(), 7);

        for node in tree.iter_breadth_first() {
            if node.arity() == Arity::Any {
                assert_eq!(node.children().map(|c| c.len()), Some(2));
            } else if let Arity::Exact(n) = node.arity() {
                assert_eq!(node.children().map(|c| c.len()), Some(n));
            } else {
                assert_eq!(node.children(), None);
            }
        }
    }

    #[test]
    fn test_tree_builder_depth_three() {
        // just a quality of life test to make sure the builder is working.
        // The above test should be good enough, but just for peace of mind.
        let store = vec![
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let tree = Tree::with_depth(3, store);

        assert!(tree.root().is_some());
        assert_eq!(tree.root().unwrap().children().unwrap().len(), 2);
        assert_eq!(tree.height(), 3);
        assert_eq!(tree.size(), 15);

        for node in tree.iter_breadth_first() {
            if node.arity() == Arity::Any {
                assert_eq!(node.children().map(|c| c.len()), Some(2));
            } else if let Arity::Exact(n) = node.arity() {
                assert_eq!(node.children().map(|c| c.len()), Some(n));
            } else {
                assert_eq!(node.children(), None);
            }
        }
    }

    #[test]
    fn test_vertex_with_any_arity_builds_correct_depth() {
        let tree = Tree::with_depth(
            2,
            vec![
                (
                    NodeType::Vertex,
                    vec![Op::sigmoid(), Op::relu(), Op::tanh()],
                ),
                (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
            ],
        );

        assert!(tree.root().is_some());
        assert_eq!(tree.root().unwrap().children().unwrap().len(), 2);
        assert_eq!(tree.height(), 2);
        assert_eq!(tree.size(), 7);

        for node in tree.iter_breadth_first() {
            if node.arity() == Arity::Any {
                assert_eq!(node.children().map(|c| c.len()), Some(2));
            } else if let Arity::Exact(n) = node.arity() {
                assert_eq!(node.children().map(|c| c.len()), Some(n));
            } else {
                assert_eq!(node.children(), None);
            }
        }
    }
}
