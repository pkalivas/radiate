use crate::collections::{Tree, TreeNode};
use crate::NodeCell;
use radiate::random_provider;

pub struct TreeBuilder<C: NodeCell> {
    gates: Vec<C>,
    leafs: Vec<C>,
    depth: usize,
}

impl<C: NodeCell> TreeBuilder<C> {
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

    pub fn with_gates(mut self, gates: Vec<C>) -> Self {
        self.gates = gates;
        self
    }

    pub fn with_leafs(mut self, leafs: Vec<C>) -> Self {
        self.leafs = leafs;
        self
    }

    pub fn build(&self) -> Tree<C>
    where
        C: Default,
    {
        let root = self.grow_tree(self.depth);
        Tree::new(root)
    }

    fn grow_tree(&self, depth: usize) -> TreeNode<C>
    where
        C: Default,
    {
        if depth == 0 {
            let leaf = if self.leafs.is_empty() {
                C::default()
            } else {
                random_provider::choose(&self.leafs).new_instance()
            };

            return TreeNode::new(leaf);
        }

        let gate = if self.gates.is_empty() {
            C::default()
        } else {
            random_provider::choose(&self.gates).new_instance()
        };

        let mut parent = TreeNode::new(gate);
        for _ in 0..*parent.value().arity() {
            let temp = self.grow_tree(depth - 1);
            parent.add_child(temp);
        }

        parent
    }
}
