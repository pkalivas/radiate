use uuid::Uuid;

use super::*;
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

/// A tree data structure that can be used to represent a tree of nodes.
///
/// # Type Parameters
/// - `T`: The type of the value stored in each node.
///
#[derive(Default)]
pub struct Tree<T>
where
    T: Clone + PartialEq + Default,
{
    root: Option<TreeNode<T>>,
}

impl<T> Tree<T>
where
    T: Clone + PartialEq + Default,
{
    /// Creates a new tree with the given root node.
    pub fn new(root: TreeNode<T>) -> Self {
        Tree { root: Some(root) }
    }

    pub fn in_order_iter(&self) -> impl Iterator<Item = &TreeNode<T>> {
        iterators::TreeInOrderIter::new(self)
    }

    pub fn in_order_iter_mut(&mut self) -> impl Iterator<Item = &mut TreeNode<T>> {
        iterators::TreeInorderIterMut::new(self)
    }

    pub fn root(&self) -> Option<&TreeNode<T>> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut TreeNode<T>> {
        self.root.as_mut()
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
                    .map(|child| calc_depth(&child))
                    .max()
                    .unwrap_or(0)
            }
        }

        self.root
            .as_ref()
            .map(|root| calc_depth(&root))
            .unwrap_or(0)
    }

    /// Get the size of the tree. The size of a tree is the number of nodes in the tree.
    pub fn size(&self) -> usize {
        fn calc_size<T: Clone + PartialEq + Default>(node: &TreeNode<T>) -> usize {
            1 + node
                .children()
                .iter()
                .map(|child| calc_size(&child))
                .sum::<usize>()
        }

        self.root.as_ref().map(|root| calc_size(&root)).unwrap_or(0)
    }

    /// Given a target node id, return the sub tree rooted at the target node.
    pub fn sub_tree(&self, target_id: Uuid) -> Option<Tree<T>> {
        fn find_node<T: Clone + PartialEq + Default>(
            node: &TreeNode<T>,
            target_id: Uuid,
        ) -> Option<TreeNode<T>> {
            if node.id() == target_id {
                return Some(node.clone());
            }
            for child in node.children() {
                if let Some(found) = find_node(&child, target_id) {
                    return Some(found);
                }
            }
            None
        }

        self.root
            .as_ref()
            .and_then(|root| find_node(&root, target_id).map(|node| Tree::new(node)))
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
                    find_node(child, target_id, new_sub_tree.clone());
                }
            }
        }

        if let Some(root) = self.root.as_mut() {
            find_node(root, target_id, new_sub_tree);
        }
    }

    pub fn with_depth<F>(depth: usize, f: F) -> Self
    where
        F: Fn(usize, Option<&TreeNode<T>>) -> Vec<TreeNode<T>>,
    {
        if depth == 0 {
            return Tree::default();
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
        Tree::new(root)
    }
}

impl<T> NodeCollectionTwo for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    type Node = TreeNode<T>;

    fn from_nodes(nodes: Vec<TreeNode<T>>) -> Self {
        Tree::new(nodes.iter().next().unwrap().clone())
    }

    fn get(&self, index: usize) -> &Self::Node {
        if index == 0 {
            return self.root().unwrap();
        } else {
            let mut cnt = index;
            for node in self.in_order_iter() {
                if cnt == 0 {
                    return node;
                }
                cnt -= 1;
            }
        }

        panic!("Index out of bounds for tree with {} nodes", self.size());
    }

    fn get_mut(&mut self, index: usize) -> &mut Self::Node {
        if index == 0 {
            return self.root_mut().unwrap();
        } else {
            let mut cnt = index;

            for node in self.in_order_iter_mut() {
                if cnt == 0 {
                    return node;
                }
                cnt -= 1;
            }
        }

        panic!("Index out of bounds for tree with ");
    }

    fn iter(&self) -> impl Iterator<Item = &Self::Node> {
        self.in_order_iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Node> {
        self.in_order_iter_mut()
    }
}

impl<T> Index<usize> for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    type Output = TreeNode<T>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T> IndexMut<usize> for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<T> Valid for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        true
    }
}

impl<T> Clone for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn clone(&self) -> Self {
        Tree {
            root: self.root.as_ref().map(|root| root.clone()),
        }
    }
}

impl<T> From<TreeNode<T>> for Tree<T>
where
    T: Clone + PartialEq + Default,
{
    fn from(node: TreeNode<T>) -> Self {
        Tree::new(node)
    }
}

impl<T> Display for Tree<T>
where
    T: Clone + PartialEq + Default + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_node<T: Clone + PartialEq + Default + Display>(
            node: &TreeNode<T>,
            f: &mut std::fmt::Formatter<'_>,
            depth: usize,
        ) -> std::fmt::Result {
            writeln!(
                f,
                "{:indent$}{}",
                "",
                node.cell.borrow().value(),
                indent = depth * 2
            )?;
            for child in node.children() {
                print_node(&child, f, depth + 1)?;
            }
            Ok(())
        }

        if let Some(root) = self.root.as_ref() {
            print_node(&root, f, 0)
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
        let tree = Tree::new(root);

        assert_eq!(tree.depth(), 1);
    }

    #[test]
    fn test_tree_two_depth_two() {
        let mut root = TreeNode::<Ops<f32>>::new(op::add());
        let child = TreeNode::<Ops<f32>>::new(op::add());
        root.add_child(child);

        let tree = Tree::new(root);

        assert_eq!(tree.depth(), 2);
    }

    #[test]
    fn tree_two_with_depth_produces_correct_tree() {
        let values = (0..10).collect::<Vec<_>>();
        const DEPTH: usize = 3;
        const CHILDREN: usize = 2;

        let tree = Tree::with_depth(DEPTH, |_, _| {
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

        let tree = Tree::with_depth(DEPTH, |depth, parent: Option<&TreeNode<Ops<f32>>>| {
            let mut children = Vec::new();
            if let Some(parent) = parent {
                for _ in 0..parent.cell.borrow().value().arity() {
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

// use radiate::Valid;

// use super::BreadthFirstIterator;
// use crate::node::Node;
// use crate::{
//     node_collections, schema::collection_type::CollectionType, NodeCollection, NodeFactory,
//     NodeRepairs,
// };

// #[derive(Clone, PartialEq, Default)]
// pub struct Tree<T>
// where
//     T: Clone + PartialEq,
// {
//     pub nodes: Vec<Node<T>>,
// }

// impl<T> Tree<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub fn new(nodes: Vec<Node<T>>) -> Self {
//         Tree { nodes }
//     }

//     pub fn sub_tree(&self, index: usize) -> Self {
//         let nodes = BreadthFirstIterator::new(&self.nodes, index).collect::<Vec<&Node<T>>>();

//         Tree::new(node_collections::reindex(0, nodes.as_slice()))
//     }
// }

// impl<T> NodeCollection<T> for Tree<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     fn from_nodes(nodes: Vec<Node<T>>) -> Self {
//         Self { nodes }
//     }

//     fn get(&self, index: usize) -> &Node<T> {
//         self.nodes.get(index).unwrap_or_else(|| {
//             panic!(
//                 "Node index {} out of bounds for tree with {} nodes",
//                 index,
//                 self.nodes.len()
//             )
//         })
//     }

//     fn get_mut(&mut self, index: usize) -> &mut Node<T> {
//         let length = self.nodes.len();
//         self.nodes.get_mut(index).unwrap_or_else(|| {
//             panic!(
//                 "Node index {} out of bounds for tree with {} nodes",
//                 index, length
//             )
//         })
//     }

//     fn get_nodes(&self) -> &[Node<T>] {
//         &self.nodes
//     }

//     fn get_nodes_mut(&mut self) -> &mut [Node<T>] {
//         &mut self.nodes
//     }
// }

// impl<T> NodeRepairs<T> for Tree<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     fn repair(&mut self, _: Option<&NodeFactory<T>>) -> Self {
//         let mut collection = self.clone();

//         for node in collection.iter_mut() {
//             node.collection_type = Some(CollectionType::Tree);
//         }

//         collection
//     }
// }

// impl<T> Valid for Tree<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     fn is_valid(&self) -> bool {
//         self.nodes.iter().all(|node| node.is_valid())
//     }
// }

// impl<T> std::fmt::Debug for Tree<T>
// where
//     T: Clone + PartialEq + Default + std::fmt::Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Tree {{\n")?;
//         for node in self.get_nodes() {
//             write!(f, "  {:?},\n", node)?;
//         }
//         write!(f, "}}")
//     }
// }
