use crate::collections::{Tree, TreeNode};
use crate::node::Node;
use crate::{Arity, Factory, NodeStore, NodeType};

impl<T> Tree<T> {
    /// Create a tree with the given depth, where each node is a random node from the node store.
    /// This obey's the rules of the `NodeStore`'s `NodeType`'s arity, and will create a tree
    /// that is as balanced as possible.
    ///
    /// Note that the root node will try to be a `NodeType::Root` if it is available in the
    /// `NodeStore`, otherwise it will be a `NodeType::Vertex`. This allows caller's to specify what
    /// the root node is if desired, otherwise it will be a random vertex node from the `NodeStore`.
    ///
    /// # The `NodeStore` must contain at least one `NodeType::Root` and one `NodeType::Vertex`
    ///
    /// # Arguments
    /// * `depth` - The depth of the tree.
    /// * `nodes` - The node store to use for the tree.
    ///
    /// # Returns
    /// A tree with the given depth, where each node is a random node from the node store.
    pub fn with_depth(depth: usize, nodes: impl Into<NodeStore<T>>) -> Self
    where
        T: Default + Clone,
    {
        let store = nodes.into();

        let mut root = if store.contains_type(NodeType::Root) {
            store.new_instance(NodeType::Root)
        } else {
            store.new_instance(NodeType::Vertex)
        };

        if root.arity() == Arity::Any {
            for _ in 0..2 {
                root.add_child(Tree::grow(depth - 1, &store));
            }
        } else {
            for _ in 0..*root.arity() {
                root.add_child(Tree::grow(depth - 1, &store));
            }
        }

        Tree::new(root)
    }

    /// Recursively grow a tree from the given depth, where each node is a random node from the
    /// node store. If the depth is 0, then a leaf node is returned. Otherwise, a vertex node is
    /// returned with children that are grown from the given depth.
    /// This obey's the rules of the `NodeStore`'s `NodeType`'s arity, and will create a tree
    /// that is as balanced as possible.
    ///
    /// # Arguments
    /// * `current_depth` - The current depth of the tree.
    /// * `store` - The node store to use for the tree.
    ///
    /// # Returns
    /// A tree node with the given depth, where each node is a random node from the node store.
    fn grow(current_depth: usize, store: &NodeStore<T>) -> TreeNode<T>
    where
        T: Default + Clone,
    {
        if current_depth == 0 {
            return store.new_instance(NodeType::Leaf);
        }

        let mut parent = store.new_instance(NodeType::Vertex);
        for _ in 0..*parent.arity() {
            parent.add_child(Tree::grow(current_depth - 1, store));
        }

        parent
    }
}

#[cfg(test)]
mod tests {
    use crate::Op;

    use super::*;

    #[test]
    fn test_tree_builder_depth_two() {
        let store = vec![
            (NodeType::Root, vec![Op::add(), Op::sub()]),
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let tree = Tree::with_depth(2, store);

        println!("{:?}", tree.root().unwrap().children().unwrap().len());

        assert!(tree.root().is_some());
        assert!(tree.root().unwrap().children().unwrap().len() == 2);
        assert!(tree.height() == 2);
        assert!(tree.size() == 7);
    }

    #[test]
    fn test_tree_builder_depth_three() {
        // just a quality of life test to make sure the builder is working.
        // The above test should be good enough, but just for peace of mind.
        let store = vec![
            (NodeType::Root, vec![Op::add(), Op::sub()]),
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];
        let tree = Tree::with_depth(3, store);

        assert!(tree.root().is_some());
        assert!(tree.root().unwrap().children().unwrap().len() == 2);
        assert!(tree.height() == 3);
        assert!(tree.size() == 15);
    }
}
