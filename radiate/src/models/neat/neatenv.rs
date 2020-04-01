
use super::activation::Activation;

use crate::engine::environment::Envionment;


/// Configuation settings for the NeatAlgorithm 
/// 
/// weight_mutate_rate: the probability of uniformly perturbing a connection weight or assigning a new random value
/// weight_perturb: the uniform range to perturb a weight by will go from negative num to pos (if you enter 5, it will pertub a weight randomly between -5 and 5)
/// new_node_rate: the probability of adding a new node to the network
/// new_edge_rate: the probability of adding a new edge to the network
/// edit_weights: the probability of weights in the network being edited or just left alone
/// reactivate: the probability of reactivating a connection between two neurons 


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeatEnvironment {
    // base settings for evolution
    pub weight_mutate_rate: Option<f32>,
    pub weight_perturb: Option<f32>,
    pub new_node_rate: Option<f32>,
    pub new_edge_rate: Option<f32>,
    pub recurrent_neuron_rate: Option<f32>,
    pub edit_weights: Option<f32>,
    pub reactivate: Option<f32>,
    pub input_size: Option<u32>,
    pub output_size: Option<u32>,
    pub activation_functions: Vec<Activation>,
}


impl NeatEnvironment {

    pub fn new() -> Self {
        NeatEnvironment {
            weight_mutate_rate: None,
            weight_perturb: None,
            new_node_rate: None,
            new_edge_rate: None,
            recurrent_neuron_rate: None,
            edit_weights: None,
            reactivate: None,
            input_size: None,
            output_size: None,
            activation_functions: vec![Activation::Sigmoid],
        }
    }


    pub fn set_weight_mutate_rate(mut self, num: f32) -> Self {
        self.weight_mutate_rate = Some(num);
        self
    }


    pub fn set_weight_perturb(mut self, num: f32) -> Self {
        self.weight_perturb = Some(num);
        self
    }


    pub fn set_new_node_rate(mut self, num: f32) -> Self {
        self.new_node_rate = Some(num);
        self
    }


    pub fn set_recurrent_neuron_rate(mut self, num: f32) -> Self {
        self.recurrent_neuron_rate = Some(num);
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


    pub fn set_input_size(mut self, num: u32) -> Self {
        self.input_size = Some(num);
        self
    }


    pub fn set_output_size(mut self, num: u32) -> Self {
        self.output_size = Some(num);
        self
    }


    pub fn set_activation_functions(mut self, funcs: Vec<Activation>) -> Self {
        self.activation_functions = funcs;
        self
    }


}


unsafe impl Send for NeatEnvironment {}
unsafe impl Sync for NeatEnvironment {}


impl Default for NeatEnvironment {
    fn default() -> Self {
        Self::new()
    }
}



impl Envionment for NeatEnvironment {}