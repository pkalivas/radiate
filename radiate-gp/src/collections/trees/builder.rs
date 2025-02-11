use crate::collections::{Tree, TreeNode};
use crate::node::Node;
use crate::{Arity, Factory, NodeStore, NodeType};

impl<T> Tree<T> {
    pub fn with_depth(depth: usize, nodes: impl Into<NodeStore<T>>) -> Self
    where
        T: Default + Clone,
    {
        let store = nodes.into();

        fn grow<T>(current_depth: usize, store: &NodeStore<T>) -> TreeNode<T>
        where
            T: Default + Clone,
        {
            if current_depth == 0 {
                return store.new_instance(NodeType::Leaf);
            }

            let mut parent = store.new_instance(NodeType::Vertex);
            for _ in 0..*parent.arity() {
                parent.add_child(grow(current_depth - 1, store));
            }

            parent
        }

        let mut root = store.new_instance(NodeType::Root);

        if root.arity() == Arity::Any {
            for _ in 0..2 {
                root.add_child(grow(depth - 1, &store));
            }
        } else {
            for _ in 0..*root.arity() {
                root.add_child(grow(depth - 1, &store));
            }
        }

        Tree::new(root)
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
