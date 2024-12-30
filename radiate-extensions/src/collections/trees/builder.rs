use crate::ops::operation::Operation;

use crate::collections::{Tree, TreeNode};
use radiate::random_provider;

pub struct TreeBuilder<T> {
    gates: Vec<Operation<T>>,
    leafs: Vec<Operation<T>>,
    depth: usize,
}

impl<T> TreeBuilder<T>
where
    T: Clone,
{
    pub fn new(depth: usize) -> Self {
        TreeBuilder {
            gates: Vec::new(),
            leafs: Vec::new(),
            depth,
        }
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_gates(mut self, gates: Vec<Operation<T>>) -> Self {
        self.gates = gates;
        self
    }

    pub fn with_leafs(mut self, leafs: Vec<Operation<T>>) -> Self {
        self.leafs = leafs;
        self
    }

    pub fn build(&self) -> Tree<T>
    where
        T: Default,
    {
        let root = self.grow_tree(self.depth);
        Tree::new(root)
    }

    fn grow_tree(&self, depth: usize) -> TreeNode<T>
    where
        T: Default,
    {
        if depth == 0 {
            let leaf = if self.leafs.is_empty() {
                Operation::default()
            } else {
                random_provider::choose(&self.leafs).new_instance()
            };

            return TreeNode::new(leaf);
        }

        let gate = if self.gates.is_empty() {
            Operation::default()
        } else {
            random_provider::choose(&self.gates).new_instance()
        };

        let mut parent = TreeNode::new(gate);
        for _ in 0..*parent.value.arity() {
            let temp = self.grow_tree(depth - 1);
            parent.add_child(temp);
        }

        parent
    }
}
