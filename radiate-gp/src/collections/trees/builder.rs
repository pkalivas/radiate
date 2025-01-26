use std::sync::Arc;

use crate::collections::{Tree, TreeNode};
use crate::{Builder, Op};
use radiate::random_provider;

pub struct TreeBuilder<T> {
    depth: usize,
    gates: Vec<Op<T>>,
    leafs: Vec<Op<T>>,
    constraint: Option<Arc<Box<dyn Fn(&TreeNode<T>) -> bool>>>,
}

impl<T> TreeBuilder<T> {
    pub fn new(depth: usize) -> Self {
        TreeBuilder {
            depth,
            gates: Vec::new(),
            leafs: Vec::new(),
            constraint: None,
        }
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_gates(mut self, gates: Vec<Op<T>>) -> Self {
        self.gates = gates;
        self
    }

    pub fn with_leafs(mut self, leafs: Vec<Op<T>>) -> Self {
        self.leafs = leafs;
        self
    }

    pub fn with_constraint<F>(mut self, constraint: F) -> Self
    where
        F: Fn(&TreeNode<T>) -> bool + 'static,
    {
        self.constraint = Some(Arc::new(Box::new(constraint)));
        self
    }

    fn grow_tree(&self, depth: usize) -> TreeNode<T>
    where
        T: Default + Clone,
    {
        if depth == 0 {
            let leaf = if self.leafs.is_empty() {
                Op::default()
            } else {
                random_provider::choose(&self.leafs).clone()
            };

            return TreeNode::new(leaf);
        }

        let gate = if self.gates.is_empty() {
            Op::default()
        } else {
            random_provider::choose(&self.gates).clone()
        };

        let mut parent = TreeNode::new(gate);
        for _ in 0..*parent.value().arity() {
            let node = self.grow_tree(depth - 1);
            parent.add_child(node);
        }

        parent
    }
}

impl<T: Default + Clone> Builder for TreeBuilder<T> {
    type Output = Tree<T>;

    fn build(&self) -> Self::Output {
        let root = self.grow_tree(self.depth);
        Tree::new(root)
    }
}

#[cfg(test)]
mod tests {
    use crate::Op;

    use super::*;

    #[test]
    fn test_tree_builder_depth_two() {
        let builder = TreeBuilder::new(2)
            .with_gates(vec![Op::add(), Op::mul()])
            .with_leafs(vec![Op::value(1.0), Op::value(2.0)]);

        let tree = builder.build();

        assert!(tree.root().is_some());
        assert!(tree.root().unwrap().children().unwrap().len() == 2);
        assert!(tree.height() == 2);
        assert!(tree.size() == 7);
    }

    #[test]
    fn test_tree_builder_depth_three() {
        // just a quality of life test to make sure the builder is working.
        // The above test should be good enough, but just for peace of mind.
        let builder = TreeBuilder::new(3)
            .with_gates(vec![Op::add(), Op::mul()])
            .with_leafs(vec![Op::value(1.0), Op::value(2.0)]);

        let tree = builder.build();

        assert!(tree.root().is_some());
        assert!(tree.root().unwrap().children().unwrap().len() == 2);
        assert!(tree.height() == 3);
        assert!(tree.size() == 15);
    }
}
