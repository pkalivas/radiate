use crate::TreeNode;
use std::collections::VecDeque;
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

    pub fn iter_breadth_first(&self) -> impl Iterator<Item = &TreeNode<T>> {
        BreadthFirstIterator {
            queue: self
                .root
                .as_ref()
                .map_or(VecDeque::new(), |root| vec![root].into_iter().collect()),
        }
    }

    pub fn iter_pre_order(&self) -> impl Iterator<Item = &TreeNode<T>> {
        PreOrderIterator {
            stack: self
                .root
                .as_ref()
                .map_or(Vec::new(), |root| vec![root].into_iter().collect()),
        }
    }

    pub fn iter_post_order(&self) -> impl Iterator<Item = &TreeNode<T>> {
        PostOrderIterator {
            stack: self
                .root
                .as_ref()
                .map_or(Vec::new(), |root| vec![(root, false)].into_iter().collect()),
        }
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

pub struct PreOrderIterator<'a, T> {
    stack: Vec<&'a TreeNode<T>>,
}

impl<'a, T> Iterator for PreOrderIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            // Push children in reverse order for correct traversal
            if let Some(children) = node.children() {
                for child in children.iter().rev() {
                    self.stack.push(child);
                }
            }
            node
        })
    }
}

pub struct PostOrderIterator<'a, T> {
    stack: Vec<(&'a TreeNode<T>, bool)>,
}

impl<'a, T> Iterator for PostOrderIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, visited)) = self.stack.pop() {
            if visited {
                return Some(node);
            }
            self.stack.push((node, true));
            if let Some(children) = node.children() {
                for child in children.iter().rev() {
                    self.stack.push((child, false));
                }
            }
        }
        None
    }
}

pub struct BreadthFirstIterator<'a, T> {
    queue: VecDeque<&'a TreeNode<T>>,
}

impl<'a, T> Iterator for BreadthFirstIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        if let Some(children) = node.children() {
            self.queue.extend(children.iter());
        }
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expr, NodeCell, NodeType};

    #[test]
    fn test_tree_traversal() {
        // Create a simple tree:
        //       1
        //      / \
        //     2   3
        //    /
        //   4
        let leaf = NodeCell::new(expr::value(4.0), NodeType::Leaf);
        let node2 = TreeNode::with_children(
            NodeCell::new(expr::value(2.0), NodeType::Gate),
            vec![TreeNode::new(leaf)],
        );

        let node3 = TreeNode::new(NodeCell::new(expr::value(3.0), NodeType::Leaf));

        let root = Tree::new(TreeNode::with_children(
            NodeCell::new(expr::value(1.0), NodeType::Gate),
            vec![node2, node3],
        ));

        // Test pre-order
        let pre_order: Vec<f32> = root
            .iter_pre_order()
            .map(|n| match &n.cell.value {
                expr::Expr::Value(v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(pre_order, vec![1.0, 2.0, 4.0, 3.0]);

        // Test post-order
        let post_order: Vec<f32> = root
            .iter_post_order()
            .map(|n| match &n.cell.value {
                expr::Expr::Value(v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(post_order, vec![4.0, 2.0, 3.0, 1.0]);

        // Test breadth-first
        let bfs: Vec<f32> = root
            .iter_breadth_first()
            .map(|n| match &n.cell.value {
                expr::Expr::Value(v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(bfs, vec![1.0, 2.0, 3.0, 4.0]);
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
