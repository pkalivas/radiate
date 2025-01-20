use crate::collections::{Tree, TreeNode};
use crate::{Builder, NodeCell};
use radiate::random_provider;

pub struct TreeBuilder<C: NodeCell> {
    depth: usize,
    gates: Vec<C>,
    leafs: Vec<C>,
}

impl<C: NodeCell> TreeBuilder<C> {
    pub fn new(depth: usize) -> Self {
        TreeBuilder {
            depth,
            gates: Vec::new(),
            leafs: Vec::new(),
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
            let node = self.grow_tree(depth - 1);
            parent.add_child(node);
        }

        parent
    }
}

impl<C: NodeCell + Default> Builder for TreeBuilder<C> {
    type Output = Tree<C>;

    fn build(&self) -> Self::Output {
        let root = self.grow_tree(self.depth);
        Tree::new(root)
    }
}

impl<C: NodeCell + Default> Default for TreeBuilder<C> {
    fn default() -> Self {
        TreeBuilder::new(1)
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
