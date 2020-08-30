use std::fmt;
use std::marker::Sync;
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use rand::Rng;

pub mod node;
pub mod iterators;

pub use node::{Node, Link};


/// A tree struct to encapsulate a bidirectional binary tree.AsMut
/// 
/// Each node within the tree has three pointers to its parent, left, and right child. 
/// 
/// This struct holds the root of the tree. The tree also contains a size which represents the number of nodes in the tree,
/// an input size which is the size of the input vector (1D), and is used to generate nodes alone with the 
/// output options which is an owned vec of i32s representing different outputs of the classification.
#[derive(PartialEq)]
pub struct Tree<T: Clone> {
    root: Link<T>,
    size: i32,
}

/// implement the tree
impl<T: Clone> Tree<T> {
    /// Create a new default Tree given an input size and a vec of possible outputs 
    /// 
    /// Returns the newly created Tree with no root node, a size of 
    /// 0 and an owned input_size and output_options.
    pub fn new() -> Self {
        Tree {
            root: None,
            size: 0,
        }
    }

    /// build tree from node slice
    pub fn from_slice(nodes: &mut [Option<T>]) -> Self {
        let mut tree = Self::new();
        tree.size = nodes.len() as i32;
        tree.set_root(tree.make_tree(nodes));
        tree
    }

    pub(crate) fn root_mut_opt(&mut self) -> Option<&mut Node<T>> {
        self.root.as_mut().map(|n| n.as_mut())
    }

    pub(crate) fn root_opt(&self) -> Option<&Node<T>> {
        self.root.as_ref().map(|n| n.as_ref())
    }

    pub(crate) fn set_root(&mut self, root: Link<T>) {
        self.drop_root();
        self.root = root;
    }

    fn drop_root(&mut self) {
        self.root = None;
    }

    /// return an in order iterator which 
    /// allows for the nodes in the tree to be
    /// mutated while iterating
    pub fn iter_mut(&mut self) -> iterators::IterMut<'_, T> {
        iterators::IterMut::new(self.root_mut_opt())
    }

    /// Return a level order iterator
    /// Iterators over the tree from top to bottom, 
    /// going from parent, to it's left child then it's right child.
    /// Each node that the iterator yields is a reference to a Node struct
    pub fn level_order_iter(&self) -> iterators::LevelOrderIterator<'_, T> {
        iterators::LevelOrderIterator::new(self.root_opt())
    }

    /// Return an in order iterator
    /// Iterates over the tree in order, from left to right
    /// with the root in the middle (assuming balanced)
    /// Each node that the iterator yields is a reference to a Node struct
    pub fn in_order_iter(&self) -> iterators::InOrderIterator<'_, T> {
        iterators::InOrderIterator::new(self.root_opt())
    }

    /// the len of the tree is it's size, the numbr of nodes
    pub fn len(&self) -> usize {
        self.size as usize
    }

    /// Update size from root node.
    pub fn update_size(&mut self) {
        self.size = match self.root_opt() {
            Some(root) => root.size(),
            None => 0,
        };
    }

    /// return the height of the tree from the root 
    #[inline]    
    pub fn height(&self) -> i32 {
        match self.root_opt() {
            Some(root) => root.height(),
            None => 0,
        }
    }

    /// Get a node's element from the tree at a given index and return an option with
    /// either the node's element in it, or none.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.in_order_iter().nth(index).map(|n| n.get())
    }

    /// Get a node's element from the tree at a given index and return an option with
    /// either the node's element in it, or none.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_node_mut(index).map(|n| n.get_mut())
    }

    /// Get a node from the tree at a given index and return an option with
    /// either the node in it, or none.
    #[inline]
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        self.iter_mut().nth(index)
    }

    /// Get the index of a given node in the tree
    #[inline]    
    pub fn index_of(&self, node: &Node<T>) -> usize {
        let mut temp: Option<usize> = None;
        for (index, curr) in self.in_order_iter().enumerate() {
            if curr == node {
                temp = Some(index);
                break;
            }
        }
        temp.unwrap_or_else(|| panic!("Node index not found."))
    }

    /// Insert a node to the tree randomly and increase the size by 1.
    pub fn insert_random(&mut self, elem: T) {
        let node = Node::new(elem);
        match self.root_mut_opt() {
            Some(root) => {
                root.insert_random(node);
            },
            None => {
                self.set_root(Some(node));
            },
        }
        self.size += 1;
    }

    /// display the tree by calling the recursive display method within the node 
    /// implementation at level 0. If no root node, panic!
    pub fn display(&self) {
        match &self.root {
            None => panic!("The no root node"),
            Some(root) => root.display(0),
        }
    }

    /// Balance the tree by thin copying each node then calling a private
    /// recursive function to build the tree structure.
    /// 
    /// Return an option in order to use '?' instead of 'unwrap()' in the 
    /// function body.
    pub fn balance(&mut self) {
        let mut node_bag = self.in_order_iter()
            .map(|x: &Node<T>| Some(x.get().clone()))
            .collect::<Vec<_>>();
        self.set_root(self.make_tree(&mut node_bag[..]));
    }

    /// Recursively build a balanced binary tree by splitting the slice into left/right
    /// sides at the middle node.
    /// Return a `Link` to the middle node to be set as the child of a parent node or as the root node.
    #[inline]    
    fn make_tree(&self, bag: &mut [Option<T>]) -> Link<T> {
        let midpoint = bag.len() / 2;
        // split at midpoint
        let (left, right) = bag.split_at_mut(midpoint);
        // 'right' side has the node we need.
        if let Some((node, right)) = right.split_first_mut() {
            // take the node from the bag.  This replaces it with `None`
            let mut curr_node = Node::new(node.take().unwrap());
            // make sure it doesn't have a parent.
            curr_node.set_left_child(self.make_tree(left));
            curr_node.set_right_child(self.make_tree(right));
            Some(curr_node)
        } else {
            // bag is empty
            return None;
        }
    }

    /// get a vec of node references in a bised sense where
    /// nodes at a lower level are favored 
    #[inline]    
    pub fn get_biased_level<'a>(&'a self) -> Vec<&'a Node<T>> {
        let mut r = rand::thread_rng();
        let height = self.height();
        let index = r.gen_range(0, self.len()) as usize;
        let levels = self.level_order_iter()
            .map(|x: &Node<T>| height - x.height())
            .collect::<Vec<_>>();

        // return a vec where the depth of a node is equal to 
        // the biased level chosen. Order does not matter
        // because there will be more numbers in the levels vec with 
        // a lower depth inherintly due to tree structures
        self.in_order_iter()
            .filter(|x| x.depth() == levels[index])
            .collect::<Vec<_>>()
    }

    /// Get a biased random node from the tree by gathering a biased random level
    /// towards the bottom of the tree, then returning a reference to the chosen node
    pub fn get_biased_random_node<'a>(&'a self) -> &'a Node<T> {
        let mut nodes = self.get_biased_level();
        let index = rand::thread_rng().gen_range(0, nodes.len());
        nodes.remove(index)
    }

    /// take in an index of the tree to swap with the pointer of another subtree
    /// by simply switching the pointers of the node at swap_index and the other_node pointer
    pub(crate) fn replace(&mut self, swap_index: usize, mut other_node: Box<Node<T>>) {
        let swap_node = self.get_node_mut(swap_index)
          .expect("Index not found in tree.");
        match swap_node.is_left_child() {
            Some(true) => {
                if let Some(parent) = swap_node.parent_mut_opt() {
                    parent.set_left_child(Some(other_node));
                }
            },
            Some(false) => {
                if let Some(parent) = swap_node.parent_mut_opt() {
                    parent.set_right_child(Some(other_node));
                }
            },
            None => {
                other_node.remove_from_parent();
                self.set_root(Some(other_node));
            }
        };
        self.update_size();
    }

    /// Shuffle the tree by gathering a list of the nodes then shuffling the list
    /// and then balancing the tree again from that list
    #[inline]    
    pub fn shuffle_tree(&mut self, r: &mut ThreadRng) {
        let mut node_list = self.in_order_iter()
            .map(|x: &Node<T>| Some(x.get().clone()))
            .collect::<Vec<_>>();
        node_list.shuffle(r);
        self.set_root(self.make_tree(&mut node_list[..]));
    }
}

/// implemented a display function for the Tree just for easier debugging 
impl<T: Clone> fmt::Debug for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tree=[{}]", self.size)
    }
}

/// Return a new copy of the tree, calling deep copy from the root node and copying over
/// the size, input size, and output options in the most efficient way.
impl<T: Clone> Clone for Tree<T> {
    #[inline]
    fn clone(&self) -> Self {
        // Deep copy root node if any.
        let root = self.root_opt().map(|n| n.deepcopy());
        Tree {
            root,
            size: self.size,
        }
    }
}

/// Because the tree is made out of raw mutable pointers, if those pointers
/// are not dropped, there is a severe memory leak, like possibly gigs of
/// ram over only a few generations depending on the size of the generation
/// This drop implementation will recursively drop all nodes in the tree
impl<T: Clone> Drop for Tree<T> {
    fn drop(&mut self) { 
        self.drop_root();
    }
}

/// These must be implemented for the tree or any type to be
/// used within separate threads. Because implementing the functions
/// themselves is dangerous and unsafe and i'm not smart enough
/// to do that from scratch, these "implementations" will get rid
/// of the error and realistically they don't need to be implemented for the
/// program to work
unsafe impl<T: Clone> Send for Tree<T> {}
unsafe impl<T: Clone> Sync for Tree<T> {}

/// implement a function for getting a base default Tree which is completely empty
/// There are multiple places within the struct implementation which will panic! if 
/// this default Tree is passed through it.
impl<T: Clone> Default for Tree<T> {
    fn default() -> Self {
        Tree {
            root: None,
            size: 0
        }
    }
}
