
extern crate rand;
extern crate simple_matrix;

use std::fmt;
use std::ptr;
use std::mem;
use std::marker::Sync;
use std::sync::{Arc, RwLock};
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use rand::Rng;
use simple_matrix::Matrix;
use super::{
    iterators,
    node::Node, 
    network::NeuralNetwork, 
    evenv::TreeEnvionment
};

use crate::engine::genome::{Genome};




/// A tree struct to encapsulate a bidirectional binary tree.AsMut
/// 
/// Each node within the tree has three pointers to its parent, left, and right child. 
/// They also have a randomly generated neural network and an output option (classification).
/// 
/// This struct holds the root of the tree as a raw mutable pointer and thus this structure
/// is enherintly unsafe, however most if not all of that funtionality is encapsulated within the 
/// implementation. The tree also contains a size which represents the number of nodes in the tree,
/// an input size which is the size of the input vector (1D), and is used to generate nodes alone with the 
/// output options which is an owned vec of i32s represnting different outputs of the classification.
#[derive(PartialEq)]
pub struct Evtree {
    root: *mut Node,
    size: i32,
}



/// implement the tree
impl Evtree {

    /// Create a new default Tree given an input size and a vec of possible outputs 
    /// 
    /// Returns the newly created Tree with a null raw mutable pointer as a root, a size of 
    /// 0 and an owned input_size and output_options.
    pub fn new() -> Self {
        Evtree {
            root: ptr::null_mut(),
            size: 0,
        }
    }



    /// return an in order iterator which 
    /// allows for the nodes in the tree to be
    /// mutatued while iterating
    pub fn iter_mut(&mut self) -> iterators::IterMut {
        let mut stack = Vec::new();
        unsafe { stack.push(&mut *self.root); }
        iterators::IterMut { stack }
    }



    /// Return a level order iterator
    /// Iterators over the tree from top to bottom, 
    /// going from parent, to it's left child then it's right child.
    /// Each node that the iterator yields is a reference to a Node struct
    pub fn level_order_iter(&self) -> iterators::LevelOrderIterator {
        let mut stack = Vec::new();
        unsafe { stack.push(&*self.root); }
        iterators::LevelOrderIterator { stack }
    }



    /// Return an in order iterator
    /// Iterates over the tree in order, from left to right
    /// with the root in the middle (assuming balanced)
    /// Each node that the iterator yields is a reference to a Node struct
    pub fn in_order_iter(&self) -> iterators::InOrderIterator {
        let mut stack = Vec::new();
        unsafe { stack.push(&*self.root); }
        iterators::InOrderIterator { stack }
    }



    /// the len of the tree is it's size, the numbr of nodes
    pub fn len(&self) -> &i32 {
        &self.size
    }



    /// return the height of the tree from the root 
    #[inline]    
    pub fn height(&self) -> i32 {
        unsafe { (&*self.root).height() }
    }



    /// Get a node from the tree at a given index and return an option with
    /// either the node in it, or none. 
    /// 
    /// Panic! if the index is greater than the size of the tree.
    #[inline]    
    pub fn get(&mut self, index: usize) -> &mut Node {
        let mut temp: Option<&mut Node> = None;
        for (i, node) in self.iter_mut().enumerate() {
            if i == index {
                temp = Some(node);
                break;
            }
        }
        temp.unwrap_or_else(|| panic!("Index not found in tree."))
    }



    /// Get the index of a given node in the tree
    #[inline]    
    pub fn index_of(&self, node: &Node) -> usize {
        let mut temp: Option<usize> = None;
        for (index, curr) in self.in_order_iter().enumerate() {
            if curr == node {
                temp = Some(index);
                break;
            }
        }
        temp.unwrap_or_else(|| panic!("Node index not found."))
    }



    /// Insert a node to the tree randomly extracting away all of the unsafe code 
    /// required to iterate through the tree and check for null raw mutable pointers
    /// and increase the size by 1 each time.
    pub fn insert_random(&mut self, input_size: i32, outputs: &Vec<i32>) {
        if self.root == ptr::null_mut() {
            self.root = Node::new(input_size, outputs).as_mut_ptr();
        } else {
            unsafe { 
                (*self.root).insert_random(input_size, outputs); 
            }
        }
        self.size += 1;
    }



    /// display the tree by calling the recursive display method within the node 
    /// implementation at level 0. If the root is a null raw mutable pointer, panic!
    pub fn display(&self) {
        if self.root == ptr::null_mut() {
            panic!("The root node is ptr::null_mut()");
        }
        unsafe { (*self.root).display(0); }
    }



    /// Balance the tree by thin copying each node then calling a private
    /// recursive function to build the tree structure.
    /// 
    /// Return an option in order to use '?' instead of 'unwrap()' in the 
    /// function body.
    pub fn balance(&mut self) {
        let node_bag = self.in_order_iter()
            .map(|x: &Node| x.copy().as_mut_ptr())
            .collect::<Vec<_>>();
        self.root = self.make_tree(&node_bag[..], None)
            .unwrap_or_else(|| panic!("Tree failed to balance"));
    }



    /// Recursively build a balanced binary tree by splitting the size of the borrowed 
    /// slice of raw mutable node pointers and passing alone the parent as an option because 
    /// the parent can be null (think root node). 
    /// Return an option of a raw mutable node pointer, if None then return up a ptr::null_mut()
    #[inline]    
    fn make_tree(&self, bag: &[*mut Node], parent: Option<&*mut Node>) -> Option<*mut Node> {
        if bag.len() == 0 {
            return Some(ptr::null_mut());
        }
        let midpoint = bag.len() / 2;
        let curr_node = bag[midpoint];
        unsafe {
            (*curr_node).parent = if let Some(node) = parent { *node } else { ptr::null_mut() };  
            (*curr_node).left_child = self.make_tree(&bag[..midpoint], Some(&curr_node))?;
            (*curr_node).right_child = self.make_tree(&bag[midpoint + 1..], Some(&curr_node))?;
        }            
        Some(curr_node)
    }



    /// get a vec of node references in a bised sense where
    /// nodes at a lower level are favored 
    #[inline]    
    pub fn get_biased_level<'a>(&'a self) -> Vec<&'a Node> {
        let mut r = rand::thread_rng();
        let index = r.gen_range(0, self.len()) as usize;
        let levels = self.level_order_iter()
            .map(|x: &Node| self.height() - x.height())
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
    pub fn get_biased_random_node<'a>(&'a self) -> &'a Node {
        let mut nodes = self.get_biased_level();
        let index = rand::thread_rng().gen_range(0, nodes.len());
        nodes.remove(index)
    }



    /// take in an index of the tree to swap with the pointer of another subtree
    /// by simply switching the pointers of the node at swap_index and the other_node pointer
    fn replace(&mut self, swap_index: usize, other_node: *mut Node) {
        let swap_node = self.get(swap_index);
        unsafe {
            if !swap_node.has_parent() {
                (*other_node).parent = ptr::null_mut();
                self.root = other_node;
            } else {
                let parent = &*(swap_node).parent;
                if parent.check_left_child(swap_node) {
                    (*other_node).parent = swap_node.parent;
                    (*swap_node.parent).left_child = other_node;
                } else if parent.check_right_child(swap_node) {
                    (*other_node).parent = swap_node.parent;
                    (*swap_node.parent).right_child = other_node;
                }
            }
            self.size = (*self.root).size();
        }
    }



    /// Gut a random node from the tree. Get a random index from the tree
    /// then give that node a new neural network.
    pub fn gut_random_node(&mut self, r: &mut ThreadRng) {
        let index = r.gen_range(0, self.len()) as usize;
        let temp_node = self.get(index);
        temp_node.neural_network = NeuralNetwork::new(temp_node.input_size);
    }



    /// Shuffel the tree by gathering a list of the nodes then shuffling the list
    /// and then balancing the tree again from that list
    #[inline]    
    pub fn shuffle_tree(&mut self, r: &mut ThreadRng) {
        let mut node_list = self.in_order_iter()
            .map(|x: &Node| x.copy().as_mut_ptr())
            .collect::<Vec<_>>();
        node_list.shuffle(r);
        self.root = self.make_tree(&node_list[..], None)
            .unwrap_or_else(|| panic!("Make tree failed"));
    }



    /// Go through each of the nodes in the tree and randomly mutate 
    /// the weights and biases within the network 
    #[inline]    
    pub fn edit_random_node_networks(&mut self, weight_mutate: f32, weight_transform: f32, layer_mutate: f32) {
        for node in self.iter_mut() {
            node.neural_network.edit_weights(weight_mutate, weight_transform, layer_mutate);
        }
    }



    /// Compute the asymmetry for a single tree 
    /// by adding the height times the neural network 
    /// weight sum of the tree and putting it through the sine 
    /// function to compress the number between (-1, 1)
    #[inline]
    pub fn asymmetry(&self) -> f32 {
        let mut total: f32 = 0.0;
        for node in self.in_order_iter() {
            total += node.height() as f32 * node.neural_network.weight_sum();
        }
        total.sin()
    }



    pub fn propagate(&self, inputs: Matrix<f32>) -> u8 {
        unsafe {
            let mut curr_node = self.root;
            loop {
                let node_output = (*curr_node).neural_network.feed_forward(inputs.clone());
                let (mut max_index, mut temp_value) = (0, None);
                for i in 0..node_output.len() {
                    if node_output[i] > node_output[max_index] || temp_value.is_none() {
                        max_index = i;
                        temp_value = Some(node_output[i]);
                    }
                }

                if (&*curr_node).is_leaf() {
                    return (&*curr_node).output;
                } else if max_index == 0 && (&*curr_node).has_left_child() {
                    curr_node = (&*curr_node).left_child;
                } else if max_index == 0 && !(&*curr_node).has_left_child() {
                    curr_node = (&*curr_node).right_child;
                } else if max_index == 1 && (&*curr_node).has_right_child() {
                    curr_node = (&*curr_node).right_child;
                } else if max_index == 1 && !(&*curr_node).has_right_child() {
                    curr_node = (&*curr_node).left_child;
                } 
            }
        }
    }


}





/// implemented a display function for the Tree just for easier debugging 
impl fmt::Debug for Evtree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let address: u64 = mem::transmute(self);
            let root: u64 = if self.root != ptr::null_mut() { mem::transmute(&*self.root) } else { 0x64 };
            write!(f, "Tree=[{}, {}, {}]", address, root, self.size)
        }
    }
}
/// Return a new copy of the tree, calling deep copy from the root node and copying over
/// the size, input size, and output options in the most effecient way.
/// TODO: this looks like messy.. is there a way to clean up?
impl Clone for Evtree {
    #[inline]
    fn clone(&self) -> Evtree {
        Evtree {
            root: unsafe { (&*self.root).deepcopy() },
            size: self.size,
        }
    }
}
/// Because the tree is made out of raw mutable pointers, if those pointers
/// are not dropped, there is a severe memory leak, like possibly gigs of
/// ram over only a few generations depending on the size of the generation
/// This drop implementation will recursivley drop all nodes in the tree 
impl Drop for Evtree {
    fn drop(&mut self) { 
        unsafe {
            let mut stack = Vec::with_capacity(*self.len() as usize);
            stack.push(self.root);
            while stack.len() > 0 {
                let curr_node = stack.pop().unwrap();
                if (&*curr_node).has_left_child() {
                    stack.push((&*curr_node).left_child);
                }
                if (&*curr_node).has_right_child() {
                    stack.push((&*curr_node).right_child);
                }
                drop(Box::from_raw(curr_node));
            }
        }
    }
}
/// These must be implemneted for the tree or any type to be 
/// used within seperate threads. Because implementing the functions 
/// themselves is dangerious and unsafe and i'm not smart enough 
/// to do that from scratch, these "implmenetaions" will get rid 
/// of the error and realistically they don't need to be implemneted for the
/// program to work
unsafe impl Send for Evtree {}
unsafe impl Sync for Evtree {}
/// implement a function for getting a base default Evtree which is completetly empty
/// There are multiple places within the struct implementation which will panic! if 
/// this default Evtree is passed through it.
impl Default for Evtree {
    fn default() -> Evtree {
        Evtree {
            root: ptr::null_mut(),
            size: 0
        }
    }
}



impl Genome<Evtree, TreeEnvionment> for Evtree {

    /// one should be the more fit Evtree and two should be the less fit Evtree.
    /// This function should attemp to produce a Evtree which is no higher than the 
    /// specified max height of a Evtree.
    #[inline]
    fn crossover(one: &Evtree, two: &Evtree, settings: &Arc<RwLock<TreeEnvionment>>, crossover_rate: f32) -> Option<Evtree> {
        let set = &*(*settings).read().unwrap();
        // make a complete copy of the more fit tree and declare a random 
        // ThreadRng type to be used for random mutations
        let mut result = one.clone();
        let mut r = rand::thread_rng();

        // make sure that the tree that will be built will be less than the 
        // specified max height of a tree in a config type
        let mut node_one = one.get_biased_random_node();
        let mut node_two = two.get_biased_random_node();
        while node_one.depth() + node_two.height() > set.max_height? {
            node_one = one.get_biased_random_node();
            node_two = two.get_biased_random_node();
        }

        // The crossover consists of either subtreeing and crossing over trees 
        // or of mutating the structure of the tree by randomly mutating the neural network
        // in random nodes, or by adding nodes, gutting nodes, or shuffling the structure of the tree
        if r.gen::<f32>() < crossover_rate {
            let node_index = one.index_of(&node_one);
            result.replace(node_index, node_two.deepcopy());
        } else {
            if r.gen::<f32>() < set.get_network_mutation_rate() {
                result.edit_random_node_networks(set.weight_mutate_rate?, set.weight_transform_rate?, set.layer_mutate_rate?);
            }
            if r.gen::<f32>() < set.node_add_rate? {
                result.insert_random(set.input_size?, set.get_outputs());
            }
            if r.gen::<f32>() < set.shuffle_rate? {
                result.shuffle_tree(&mut r);
            }
            if r.gen::<f32>() < set.gut_rate? {
                result.gut_random_node(&mut r);
            }
            result.size = unsafe { (&*result.root).size() };
        }

        // return the new tree
        Some(result)
    }

    /// Implement the base trait for the tree
    /// This provides a generic way to get a base tree for starting the evolution 
    /// process
    /// Get the base tree type and return a randomly generated base tree 
    /// created through the tree settings given to it at its new() call
    fn base(settings: &mut TreeEnvionment) -> Evtree {
        let mut result = Evtree::new();
        let nodes = (0..(2 * settings.get_max_height()) - 1)
            .map(|_| Node::new(settings.get_input_size(), settings.get_outputs()).as_mut_ptr())
            .collect::<Vec<_>>();

        result.size = nodes.len() as i32;
        result.root = result.make_tree(&nodes[..], None)
            .unwrap_or_else(|| panic!("failed to make default tree."));
        result
    }


    /// takes in a Rc<RefCell<Self in order to make it simpler for the 
    /// Generation to throw types it already has inside the function by 
    /// simplmy cloing them. This function will drop the references to
    /// the Self traits at the end of this function's scope 
    fn distance(one: &Evtree, two: &Evtree, _settings: &Arc<RwLock<TreeEnvionment>>) -> f32 {
        // return the abs value of the two tree's asymmetry
        (one.asymmetry() - two.asymmetry()).abs()
    }
}


