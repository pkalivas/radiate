
extern crate rand;
extern crate simple_matrix;
extern crate radiate;

use std::sync::{Arc, RwLock};
use rand::rngs::ThreadRng;
use rand::Rng;
use simple_matrix::Matrix;

use crate::tree::*;
use super::{
    network::NeuralNetwork, 
    evenv::TreeEnvionment
};

use radiate::engine::genome::Genome;

/// a Node struct to represent a bidirectional binary tree
/// holding pointers to the parent and two children, the left and right child
/// The node also holds an input size which is the expected size of the input vector 
/// for the neural network held within the node. The output represetns the postion of the 
/// node, meaning that if it is a leaf, it will return the output from a get output function
#[derive(Debug, Clone, PartialEq)]
pub struct NetNode {
    pub neural_network: NeuralNetwork,
    pub input_size: i32,
    pub output: u8
}

/// implement the node
impl NetNode {
    /// create a new node with a given input size and a list of possible output options 
    /// 
    /// From the list of output_options the node will choose an output,
    /// from the input_size the node will create a randomly generated 
    /// neural network.
    pub fn new(input_size: i32, output_options: &[i32]) -> Self {
        let mut r = rand::thread_rng();
        let output = output_options[r.gen_range(0, output_options.len())] as u8;
        Self {
            neural_network: NeuralNetwork::new(input_size).fill_random(),
            input_size,
            output
        }
    }
}

/// A tree struct to encapsulate a bidirectional binary tree.AsMut
/// 
/// Each node within the tree has three pointers to its parent, left, and right child. 
/// They also have a randomly generated neural network and an output option (classification).
/// 
/// This struct holds the root of the tree. The tree also contains a size which represents the number of nodes in the tree,
/// an input size which is the size of the input vector (1D), and is used to generate nodes alone with the 
/// output options which is an owned vec of i32s represnting different outputs of the classification.
pub type Evtree = Tree<NetNode>;

/// implement the tree
impl Evtree {
    /// Gut a random node from the tree. Get a random index from the tree
    /// then give that node a new neural network.
    pub fn gut_random_node(&mut self, r: &mut ThreadRng) {
        let index = r.gen_range(0, self.len()) as usize;
        let temp_node = self.get_mut(index).unwrap();
        temp_node.neural_network = NeuralNetwork::new(temp_node.input_size);
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

impl Genome<Evtree, TreeEnvionment> for Evtree {
    /// one should be the more fit Evtree and two should be the less fit Evtree.
    /// This function should attemp to produce a Evtree which is no higher than the 
    /// specified max height of a Evtree.
    #[inline]
    fn crossover(one: &Evtree, two: &Evtree, settings: Arc<RwLock<TreeEnvionment>>, crossover_rate: f32) -> Option<Evtree> {
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
                result.insert_random(NetNode::new(set.input_size?, set.get_outputs()));
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
        let mut nodes = (0..(2 * settings.get_max_height()) - 1)
            .map(|_| Some(NetNode::new(settings.get_input_size(), settings.get_outputs())))
            .collect::<Vec<_>>();

        Evtree::from_slice(&mut nodes[..])
    }


    /// takes in a Rc<RefCell<Self in order to make it simpler for the 
    /// Generation to throw types it already has inside the function by 
    /// simplmy cloing them. This function will drop the references to
    /// the Self traits at the end of this function's scope 
    fn distance(one: &Evtree, two: &Evtree, _settings: Arc<RwLock<TreeEnvionment>>) -> f32 {
        // return the abs value of the two tree's asymmetry
        (one.asymmetry() - two.asymmetry()).abs()
    }
}
