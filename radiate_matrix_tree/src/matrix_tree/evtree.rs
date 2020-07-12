
extern crate rand;
extern crate simple_matrix;
extern crate radiate;

use std::fmt;
use std::marker::Sync;
use std::sync::{Arc, RwLock};
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use rand::Rng;
use simple_matrix::Matrix;
use super::{
    iterators,
    node::{Node, Link},
    network::NeuralNetwork, 
    evenv::TreeEnvionment
};

use radiate::engine::genome::Genome;




/// A tree struct to encapsulate a bidirectional binary tree.AsMut
/// 
/// Each node within the tree has three pointers to its parent, left, and right child. 
/// They also have a randomly generated neural network and an output option (classification).
/// 
/// This struct holds the root of the tree. The tree also contains a size which represents the number of nodes in the tree,
/// an input size which is the size of the input vector (1D), and is used to generate nodes alone with the 
/// output options which is an owned vec of i32s represnting different outputs of the classification.
#[derive(PartialEq)]
pub struct Evtree {
    root: Link,
    size: i32,
}



/// implement the tree
impl Evtree {

    /// Create a new default Tree given an input size and a vec of possible outputs 
    /// 
    /// Returns the newly created Tree with no root node, a size of 
    /// 0 and an owned input_size and output_options.
    pub fn new() -> Self {
        Evtree {
            root: Link::None,
            size: 0,
        }
    }


    fn root_mut_opt(&mut self) -> Option<&mut Node> {
        self.root.as_mut().map(|n| n.as_mut())
    }

    fn root_opt(&self) -> Option<&Node> {
        self.root.as_ref().map(|n| n.as_ref())
    }

    fn set_root(&mut self, root: Link) {
        self.drop_root();
        self.root = root;
    }

    fn drop_root(&mut self) {
        self.root = Link::None;
    }

    /// return an in order iterator which 
    /// allows for the nodes in the tree to be
    /// mutatued while iterating
    pub fn iter_mut(&mut self) -> iterators::IterMut {
        iterators::IterMut::new(self.root_mut_opt())
    }



    /// Return a level order iterator
    /// Iterators over the tree from top to bottom, 
    /// going from parent, to it's left child then it's right child.
    /// Each node that the iterator yields is a reference to a Node struct
    pub fn level_order_iter(&self) -> iterators::LevelOrderIterator {
        iterators::LevelOrderIterator::new(self.root_opt())
    }



    /// Return an in order iterator
    /// Iterates over the tree in order, from left to right
    /// with the root in the middle (assuming balanced)
    /// Each node that the iterator yields is a reference to a Node struct
    pub fn in_order_iter(&self) -> iterators::InOrderIterator {
        iterators::InOrderIterator::new(self.root_opt())
    }



    /// the len of the tree is it's size, the numbr of nodes
    pub fn len(&self) -> &i32 {
        &self.size
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



    /// Insert a node to the tree randomly and increase the size by 1 each time.
    pub fn insert_random(&mut self, input_size: i32, outputs: &Vec<i32>) {
        match self.root_mut_opt() {
            Some(root) => {
                root.insert_random(input_size, outputs);
            },
            None => {
                self.set_root(Some(Node::new(input_size, outputs)));
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
            .map(|x: &Node| Some(x.copy()))
            .collect::<Vec<_>>();
        self.set_root(self.make_tree(&mut node_bag[..]));
    }


    /// Recursively build a balanced binary tree by splitting the slice into left/right
    /// sides at the middle node.
    /// Return a `Link` to the middle node to be set as the child of a parent node or as the root node.
    #[inline]    
    fn make_tree(&self, bag: &mut [Link]) -> Link {
        let midpoint = bag.len() / 2;
        // split at midpoint
        let (left, right) = bag.split_at_mut(midpoint);
        // 'right' side has the node we need.
        if let Some((node, right)) = right.split_first_mut() {
            // take the node from the bag.  This replaces it with `None`
            let mut curr_node = node.take().unwrap();
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
    fn replace(&mut self, swap_index: usize, mut other_node: Box<Node>) {
        let swap_node = self.get(swap_index);
        match swap_node.parent_mut_opt() {
            Some(parent) => {
                if parent.check_left_child(swap_node) {
                    parent.set_left_child(Some(other_node));
                } else if parent.check_right_child(swap_node) {
                    parent.set_right_child(Some(other_node));
                } else {
                    unreachable!("Invalid tree structure.  The node is not a child of it's parent.");
                }
            },
            None => {
                other_node.remove_from_parent();
                self.set_root(Some(other_node));
            }
        }
        self.update_size();
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
            .map(|x: &Node| Some(x.copy()))
            .collect::<Vec<_>>();
        node_list.shuffle(r);
        self.set_root(self.make_tree(&mut node_list[..]));
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
        let mut curr_node = self.root_opt()
            .expect("No root node.");
        loop {
            let node_output = curr_node.neural_network.feed_forward(inputs.clone());
            let (mut max_index, mut temp_value) = (0, None);
            for i in 0..node_output.len() {
                if node_output[i] > node_output[max_index] || temp_value.is_none() {
                    max_index = i;
                    temp_value = Some(node_output[i]);
                }
            }

            if curr_node.is_leaf() {
                return curr_node.output;
            } else {
                let next_node = if max_index == 0 {
                    curr_node.left_child_opt().or_else(|| {
                        curr_node.right_child_opt()
                    })
                } else {
                    curr_node.right_child_opt().or_else(|| {
                        curr_node.left_child_opt()
                    })
                };
                curr_node = next_node
                    .expect("Non-leaf node doesn't have any children.");
            }
        }
    }


}





/// implemented a display function for the Tree just for easier debugging 
impl fmt::Debug for Evtree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tree=[{}]", self.size)
    }
}
/// Return a new copy of the tree, calling deep copy from the root node and copying over
/// the size, input size, and output options in the most effecient way.
/// TODO: this looks like messy.. is there a way to clean up?
impl Clone for Evtree {
    #[inline]
    fn clone(&self) -> Evtree {
        // Deep copy root node if any.
        let root = self.root_opt().map(|n| n.deepcopy());
        Evtree {
            root,
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
        self.drop_root();
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
            root: Link::None,
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
            result.update_size();
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
        let mut nodes = (0..(2 * settings.get_max_height()) - 1)
            .map(|_| Some(Node::new(settings.get_input_size(), settings.get_outputs())))
            .collect::<Vec<_>>();

        result.size = nodes.len() as i32;
        result.set_root(result.make_tree(&mut nodes[..]));
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


