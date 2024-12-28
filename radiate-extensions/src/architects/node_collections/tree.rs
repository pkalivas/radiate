use crate::{TreeIterator, TreeNode};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Default)]
pub struct Tree<T> {
    root: Option<TreeNode<T>>,
}

impl<T> Tree<T> {
    pub fn new(root: TreeNode<T>) -> Self {
        Tree { root: Some(root) }
    }

    pub fn root(&self) -> Option<&TreeNode<T>> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut TreeNode<T>> {
        self.root.as_mut()
    }
}

impl<T> AsRef<TreeNode<T>> for Tree<T> {
    fn as_ref(&self) -> &TreeNode<T> {
        self.root.as_ref().unwrap()
    }
}

impl<T> AsMut<TreeNode<T>> for Tree<T> {
    fn as_mut(&mut self) -> &mut TreeNode<T> {
        self.root.as_mut().unwrap()
    }
}

impl<T> Debug for Tree<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree {{\n")?;
        for node in self.iter_breadth_first() {
            write!(f, "  {:?},\n", node.cell.value)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{expr, NodeCell};

    #[test]
    fn test_tree() {
        let mut tree_one = Tree::new(TreeNode::with_children(
            NodeCell::new(expr::add()),
            vec![
                TreeNode::new(NodeCell::new(expr::value(1.0))),
                TreeNode::new(NodeCell::new(expr::value(2.0))),
            ],
        ));

        let mut tree_two = Tree::new(TreeNode::with_children(
            NodeCell::new(expr::mul()),
            vec![
                TreeNode::new(NodeCell::new(expr::value(3.0))),
                TreeNode::new(NodeCell::new(expr::value(4.0))),
            ],
        ));

        // Swap the first child of each tree
        tree_one
            .as_mut()
            .swap_subtrees(&mut tree_two.as_mut(), 1, 1);

        // Verify swap using breadth-first traversal
        let values_one: Vec<_> = tree_one
            .iter_breadth_first()
            .filter_map(|n| match &n.cell.value {
                expr::Expr::Const(_, v) => Some(*v),
                _ => None,
            })
            .collect();

        assert_eq!(values_one, vec![3.0, 2.0]);
    }
}

// use radiate::Valid;

// use crate::node::Node;
// use crate::{schema::collection_type::CollectionType, NodeCollection, NodeFactory, NodeRepairs};

// #[derive(Clone, PartialEq, Default)]
// pub struct Tree<T: PartialEq> {
//     pub nodes: Vec<Node<T>>,
// }

// impl<T: PartialEq> Tree<T> {
//     pub fn new(nodes: Vec<Node<T>>) -> Self {
//         Tree { nodes }
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
