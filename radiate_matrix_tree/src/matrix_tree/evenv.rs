
extern crate radiate;

use radiate::engine::environment::Envionment;

/// unique mutations: 
///     start_height            height of the starting tree to generate
///     max height              the maximum height of a tree
///     network mutation rate   the probability of mutating the neural network inside a node
///     node add rate           the probability of adding a node to a crossovered tree
///     gut rate                the probability of re-initalizing a neural network for a tree and changing the node's output
///     shuffle rate            the probability of randomly mixing up the tree and rebalancing it - results in a balanced tree
///     layer mutate            the probability of randomly mutating the weights and biases of the current layer of the neural network 
///     weight rate             the probability of either editing the current weight, or reinitalizing it with a new random number 
///     weight transform        if the generated random number is less than weight_rate, edit that number in the layer of the neural network by multiplying it by +/- weight_transform
///
///
/// Unique mutation options for a tree object including options for mutation individual
/// nodes and their owned neural networks as well
#[derive(Debug, Clone, PartialEq)]
pub struct TreeEnvionment {
    pub input_size: Option<i32>,
    pub outputs: Option<Vec<i32>>,
    pub start_height: Option<i32>,
    pub max_height: Option<i32>, 
    pub network_mutation_rate: Option<f32>,
    pub node_add_rate: Option<f32>,
    pub gut_rate: Option<f32>,
    pub shuffle_rate: Option<f32>,
    pub layer_mutate_rate: Option<f32>,
    pub weight_mutate_rate: Option<f32>,
    pub weight_transform_rate: Option<f32>,
}






/// implement the Settings
impl TreeEnvionment {

    /// create a new Settings struct that is completely empty
    /// 
    /// Note: this is initiated with everything being none, however the 
    /// algorithm will panic! at several places if these options are not set 
    /// The only reason this is set like this is to avoid confusion by creating 
    /// a constructor with like 10 different arguments which have to be placed
    /// at the correct place within it.
    pub fn new() -> Self {
        TreeEnvionment {
            input_size: None,
            outputs: None,
            start_height: None,
            max_height: None,
            network_mutation_rate: None,
            node_add_rate: None,
            gut_rate: None,
            shuffle_rate: None,
            layer_mutate_rate: None,
            weight_mutate_rate: None,
            weight_transform_rate: None
        }
    }



    #[allow(dead_code)]
    pub fn set_input_size(mut self, size: i32) -> Self {
        self.input_size = Some(size);
        self
    }    

    #[allow(dead_code)]
    pub fn get_input_size(&self) -> i32 {
        self.input_size.unwrap_or_else(|| panic!("Input size not set"))
    }



    #[allow(dead_code)]
    pub fn set_outputs(mut self, outs: Vec<i32>) -> Self {
        self.outputs = Some(outs);
        self
    }

    #[allow(dead_code)]
    pub fn get_outputs(&self) -> &[i32] {
        self.outputs.as_ref().unwrap_or_else(|| panic!("Outputs not set"))
    }


    
    #[allow(dead_code)]
    pub fn set_start_height(mut self, height: i32) -> Self {
        self.start_height = Some(height);
        self
    }

    #[allow(dead_code)]
    pub fn get_start_height(&self) -> i32 {
        self.start_height.unwrap_or_else(|| panic!("Start height not set."))
    }



    #[allow(dead_code)]
    pub fn set_max_height(mut self, m: i32) -> Self {
        self.max_height = Some(m);
        self
    }

    #[allow(dead_code)]
    pub fn get_max_height(&self) -> i32 {
        self.max_height.unwrap_or_else(|| panic!("Max height not set."))
    }



    #[allow(dead_code)]
    pub fn set_network_mutation_rate(mut self, rate: f32) -> Self {
        self.network_mutation_rate = Some(rate);
        self
    }

    #[allow(dead_code)]
    pub fn get_network_mutation_rate(&self) -> f32 {
        self.network_mutation_rate.unwrap_or_else(|| panic!("Network mutation rate not set."))
    }



    #[allow(dead_code)]
    pub fn set_node_add_rate(mut self, rate: f32) -> Self {
        self.node_add_rate = Some(rate);
        self
    }

    #[allow(dead_code)]
    pub fn get_node_add_rate(&self) -> f32 {
        self.node_add_rate.unwrap_or_else(|| panic!("Node add rate not set."))
    }



    #[allow(dead_code)]
    pub fn set_gut_rate(mut self, rate: f32) -> Self {
        self.gut_rate = Some(rate);
        self
    }

    #[allow(dead_code)]
    pub fn get_gut_rate(&self) -> f32 {
        self.gut_rate.unwrap_or_else(|| panic!("Gut rate not set"))
    }



    #[allow(dead_code)]
    pub fn set_shuffle_rate(mut self, rate: f32) -> Self {
        self.shuffle_rate = Some(rate);
        self
    }

    #[allow(dead_code)]
    pub fn get_shuffle_rate(&self) -> f32 {
        self.shuffle_rate.unwrap_or_else(|| panic!("Shuffle rate not set."))
    }



    #[allow(dead_code)]
    pub fn set_layer_mutate_rate(mut self, rate: f32) -> Self {
        self.layer_mutate_rate = Some(rate);
        self
    }

    #[allow(dead_code)]
    pub fn get_layer_mutate_rate(&self) -> f32 {
        self.layer_mutate_rate.unwrap_or_else(|| panic!("Layer mutate rate not set"))
    }



    #[allow(dead_code)]
    pub fn set_weight_mutate_rate(mut self, rate: f32) -> Self {
        self.weight_mutate_rate = Some(rate);
        self
    }

    #[allow(dead_code)]
    pub fn get_weight_mutate_rate(&self) -> f32 {
        self.weight_mutate_rate.unwrap_or_else(|| panic!("Weight mutate rate not set."))
    }



    #[allow(dead_code)]
    pub fn set_weight_transform_rate(mut self, rate: f32) -> Self {
        self.weight_transform_rate = Some(rate);
        self
    }

    #[allow(dead_code)]
    pub fn get_weight_transform_rate(&self) -> f32 {
        self.weight_transform_rate.unwrap_or_else(|| panic!("Weight transform rate not set"))
    }

}


impl Default for TreeEnvionment {
    fn default() -> Self {
        Self::new()
    }
}

impl Envionment for TreeEnvionment {}
