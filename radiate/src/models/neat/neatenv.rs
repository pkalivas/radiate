

use std::collections::HashMap;
use super::counter::{Counter};
use super::neuron::{Neuron, Activation};
use super::edge::{Edge};

use crate::engine::environment::{Envionment};


/// Configuation settings for the NeatAlgorithm 
/// 
/// weight_mutate_rate: the probability of uniformly perturbing a connection weight or assigning a new random value
/// weight_perturb: the uniform range to perturb a weight by will go from negative num to pos (if you enter 5, it will pertub a weight randomly between -5 and 5)
/// new_node_rate: the probability of adding a new node to the network
/// new_edge_rate: the probability of adding a new edge to the network
/// edit_weights: the probability of weights in the network being edited or just left alone
/// reactivate: the probability of reactivating a connection between two neurons 
/// 
/// The C variables are used to measure the distance of networks
/// c1: excess nodes weight
/// c2: disjoint nodes weight
/// c3: Common node weight average weight



#[derive(Debug, Clone)]
pub struct NeatEnvironment {
    // base settings for evolution
    pub weight_mutate_rate: Option<f32>,
    pub weight_perturb: Option<f64>,
    pub new_node_rate: Option<f32>,
    pub new_edge_rate: Option<f32>,
    pub edit_weights: Option<f32>,
    pub reactivate: Option<f32>,
    pub c1: Option<f64>,
    pub c2: Option<f64>,
    pub c3: Option<f64>,
    pub input_size: Option<i32>,
    pub output_size: Option<i32>,
    pub activation_functions: Vec<Activation>,

    // global variables for evolution
    pub innov_counter: Counter,
    pub global_edges: HashMap<(i32, i32), Edge>,
    pub global_nodes: HashMap<(i32, i32), Neuron>
}


impl NeatEnvironment {

    pub fn new() -> Self {
        NeatEnvironment {
            weight_mutate_rate: None,
            weight_perturb: None,
            new_node_rate: None,
            new_edge_rate: None,
            edit_weights: None,
            reactivate: None,
            c1: None,
            c2: None,
            c3: None,
            input_size: None,
            output_size: None,
            activation_functions: vec![Activation::Sigmoid],
            innov_counter: Counter::new(),
            global_edges: HashMap::new(),
            global_nodes: HashMap::new()
        }
    }


    pub fn set_weight_mutate_rate(mut self, num: f32) -> Self {
        self.weight_mutate_rate = Some(num);
        self
    }


    pub fn set_weight_perturb(mut self, num: f64) -> Self {
        self.weight_perturb = Some(num);
        self
    }


    pub fn set_new_node_rate(mut self, num: f32) -> Self {
        self.new_node_rate = Some(num);
        self
    }


    pub fn set_new_edge_rate(mut self, num: f32) -> Self {
        self.new_edge_rate = Some(num);
        self
    }


    pub fn set_edit_weights(mut self, num: f32) -> Self {
        self.edit_weights = Some(num);
        self
    }


    pub fn set_reactivate(mut self, num: f32) -> Self {
        self.reactivate = Some(num);
        self
    }


    pub fn set_c1(mut self, num: f64) -> Self {
        self.c1 = Some(num);
        self
    }


    pub fn set_c2(mut self, num: f64) -> Self {
        self.c2 = Some(num);
        self
    }


    pub fn set_c3(mut self, num: f64) -> Self {
        self.c3 = Some(num);
        self
    }


    pub fn set_input_size(mut self, num: i32) -> Self {
        self.input_size = Some(num);
        self
    }


    pub fn set_output_size(mut self, num: i32) -> Self {
        self.output_size = Some(num);
        self
    }


    pub fn start_innov_counter(mut self) -> Self {
        self.innov_counter = Counter::new();
        self
    }


    pub fn set_activation_functions(mut self, funcs: Vec<Activation>) -> Self {
        self.activation_functions = funcs;
        self
    }


    pub fn get_mut_counter(&mut self) -> &mut Counter {
        &mut self.innov_counter
    }


    pub fn get_mut_nodes(&mut self) -> &mut HashMap<(i32, i32), Neuron> {
        &mut self.global_nodes
    }


    pub fn get_mut_edges(&mut self) -> &mut HashMap<(i32, i32), Edge> {
        &mut self.global_edges
    }

    
    pub fn subtract_count(&mut self, num: i32) {
        self.innov_counter.roll_back(num);
    }

}


unsafe impl Send for NeatEnvironment {}
unsafe impl Sync for NeatEnvironment {}


impl Default for NeatEnvironment {
    fn default() -> Self {
        Self::new()
    }
}



impl Envionment for NeatEnvironment {

    fn reset(&mut self) {
        self.global_edges = HashMap::new();
        self.global_nodes = HashMap::new();
    }

}