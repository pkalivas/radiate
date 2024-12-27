pub mod iterators;
pub mod tree_node;

pub use tree_node::*;
use uuid::Uuid;

use super::*;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

/// A tree data structure that can be used to represent a tree of nodes.
///
/// # Type Parameters
/// - `T`: The type of the value stored in each node.
///
#[derive(Default)]
pub struct TreeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    root: Option<Rc<RefCell<TreeNode<T>>>>,
}

impl<T> TreeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    /// Creates a new tree with the given root node.
    pub fn new(root: TreeNode<T>) -> Self {
        TreeTwo {
            root: Some(Rc::new(RefCell::new(root))),
        }
    }

    pub fn in_order_iter(&self) -> iterators::TreeInOrderIterator<T> {
        iterators::TreeInOrderIterator::new(self.root.as_ref().unwrap().clone())
    }

    /// Get the depth of the tree. The depth of a tree is the length of the longest
    /// path from the root node to a leaf node.
    pub fn depth(&self) -> usize {
        fn calc_depth<T: Clone + PartialEq + Default>(node: &TreeNode<T>) -> usize {
            if node.children.is_empty() {
                1
            } else {
                1 + node
                    .children()
                    .iter()
                    .map(|child| calc_depth(&child.borrow()))
                    .max()
                    .unwrap_or(0)
            }
        }

        self.root
            .as_ref()
            .map(|root| calc_depth(&root.borrow()))
            .unwrap_or(0)
    }

    /// Get the size of the tree. The size of a tree is the number of nodes in the tree.
    pub fn size(&self) -> usize {
        fn calc_size<T: Clone + PartialEq + Default>(node: &TreeNode<T>) -> usize {
            1 + node
                .children()
                .iter()
                .map(|child| calc_size(&child.borrow()))
                .sum::<usize>()
        }

        self.root
            .as_ref()
            .map(|root| calc_size(&root.borrow()))
            .unwrap_or(0)
    }

    /// Given a target node id, return the sub tree rooted at the target node.
    pub fn sub_tree(&self, target_id: Uuid) -> Option<TreeTwo<T>> {
        fn find_node<T: Clone + PartialEq + Default>(
            node: &TreeNode<T>,
            target_id: Uuid,
        ) -> Option<TreeNode<T>> {
            if node.id() == target_id {
                return Some(node.clone());
            }
            for child in node.children() {
                if let Some(found) = find_node(&child.borrow(), target_id) {
                    return Some(found);
                }
            }
            None
        }

        self.root
            .as_ref()
            .and_then(|root| find_node(&root.borrow(), target_id).map(|node| TreeTwo::new(node)))
    }

    pub fn swap_sub_tree(&mut self, target_id: Uuid, new_sub_tree: TreeNode<T>) {
        fn find_node<T: Clone + PartialEq + Default>(
            node: &mut TreeNode<T>,
            target_id: Uuid,
            new_sub_tree: TreeNode<T>,
        ) {
            if node.id() == target_id {
                *node = new_sub_tree;
            } else {
                for child in node.children_mut() {
                    find_node(&mut child.borrow_mut(), target_id, new_sub_tree.clone());
                }
            }
        }

        if let Some(root) = self.root.as_mut() {
            find_node(&mut root.borrow_mut(), target_id, new_sub_tree);
        }
    }

    pub fn with_depth<F>(depth: usize, f: F) -> Self
    where
        F: Fn(usize, Option<&TreeNode<T>>) -> Vec<TreeNode<T>>,
    {
        if depth == 0 {
            return TreeTwo::default();
        }

        fn build_tree<T: Clone + PartialEq + Default, F>(
            current_depth: usize,
            parent: &mut TreeNode<T>,
            f: &F,
        ) where
            F: Fn(usize, Option<&TreeNode<T>>) -> Vec<TreeNode<T>>,
        {
            if current_depth == 0 {
                return;
            }

            let children = f(current_depth, Some(parent));
            for mut child in children {
                build_tree(current_depth - 1, &mut child, f);
                parent.add_child(child);
            }
        }

        let mut root = f(depth, None).pop().unwrap();
        build_tree(depth, &mut root, &f);
        TreeTwo::new(root)
    }
}

impl<T> NodeCollectionTwo<T> for TreeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    type Node = TreeNode<T>;

    fn from_nodes(nodes: Vec<TreeNode<T>>) -> Self {
        TreeTwo::new(nodes.iter().next().unwrap().clone())
    }
}

impl<T> Valid for TreeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        true
    }
}

impl<T> Clone for TreeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    fn clone(&self) -> Self {
        TreeTwo {
            root: self
                .root
                .as_ref()
                .map(|root| Rc::new(RefCell::new(root.borrow().clone()))),
        }
    }
}

impl<T> From<TreeNode<T>> for TreeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    fn from(node: TreeNode<T>) -> Self {
        TreeTwo::new(node)
    }
}

impl<T> Display for TreeTwo<T>
where
    T: Clone + PartialEq + Default + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_node<T: Clone + PartialEq + Default + Display>(
            node: &TreeNode<T>,
            f: &mut std::fmt::Formatter<'_>,
            depth: usize,
        ) -> std::fmt::Result {
            writeln!(f, "{:indent$}{}", "", node.value(), indent = depth * 2)?;
            for child in node.children() {
                print_node(&child.borrow(), f, depth + 1)?;
            }
            Ok(())
        }

        if let Some(root) = self.root.as_ref() {
            print_node(&root.borrow(), f, 0)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::*;

    #[test]
    fn test_tree_two_depth() {
        let root = TreeNode::<Ops<f32>>::new(op::add());
        let tree = TreeTwo::new(root);

        assert_eq!(tree.depth(), 1);
    }

    #[test]
    fn test_tree_two_depth_two() {
        let mut root = TreeNode::<Ops<f32>>::new(op::add());
        let child = TreeNode::<Ops<f32>>::new(op::add());
        root.add_child(child);

        let tree = TreeTwo::new(root);

        assert_eq!(tree.depth(), 2);
    }

    #[test]
    fn tree_two_with_depth_produces_correct_tree() {
        let values = (0..10).collect::<Vec<_>>();
        const DEPTH: usize = 3;
        const CHILDREN: usize = 2;

        let tree = TreeTwo::with_depth(DEPTH, |_, _| {
            let mut children = Vec::new();
            for _ in 0..CHILDREN {
                let value = random_provider::random::<usize>() % values.len();
                let child = TreeNode::new(values[value]);
                children.push(child);
            }
            children
        });

        let expected_size = (CHILDREN.pow(DEPTH as u32 + 1) - 1) / (CHILDREN - 1);

        assert_eq!(tree.size(), expected_size);
        assert_eq!(tree.depth(), DEPTH + 1);
    }

    #[test]
    fn test_tree_two_sub_tree() {
        const DEPTH: usize = 3;
        let node_factory = NodeFactory::<f32>::regression(3);

        let tree = TreeTwo::with_depth(DEPTH, |depth, parent: Option<&TreeNode<Ops<f32>>>| {
            let mut children = Vec::new();
            if let Some(parent) = parent {
                for _ in 0..parent.value().arity() {
                    if depth == 1 {
                        let leafs = &node_factory.node_values[&NodeType::Leaf];
                        let value = random_provider::choose(&leafs);
                        let child = TreeNode::new(value.clone());
                        children.push(child);
                    } else {
                        let gates = &node_factory.node_values[&NodeType::Gate];
                        let value = random_provider::choose(&gates);
                        let child = TreeNode::new(value.clone());
                        children.push(child);
                    }
                }
            } else {
                let value = random_provider::choose(&node_factory.node_values[&NodeType::Gate]);
                let child = TreeNode::new(value.clone());
                children.push(child);
            }

            children
        });

        println!("{}", tree);

        assert_eq!(tree.depth(), DEPTH + 1);
    }
}
