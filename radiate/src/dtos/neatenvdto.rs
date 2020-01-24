
use crate::models::neat::{
    activation::Activation,
    neatenv::NeatEnvironment
};

use super::activationdto::ActivationDto;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeatEnvDto {
    pub weight_mutate_rate: Option<f32>,
    pub weight_perturb: Option<f32>,
    pub new_node_rate: Option<f32>,
    pub new_edge_rate: Option<f32>,
    pub edit_weights: Option<f32>,
    pub reactivate: Option<f32>,
    pub input_size: Option<u32>,
    pub output_size: Option<u32>,
    pub activation_functions: Vec<ActivationDto>,
}


impl NeatEnvDto {
    pub fn to_env(&self) -> NeatEnvironment {
        NeatEnvironment {
            weight_mutate_rate: self.weight_mutate_rate,
            weight_perturb: self.weight_perturb,
            new_node_rate: self.new_node_rate,
            new_edge_rate: self.new_edge_rate,
            edit_weights: self.edit_weights,
            reactivate: self.reactivate,
            input_size: self.input_size,
            output_size: self.output_size,
            activation_functions: self.activation_functions.iter()
                .map(|x| {
                    match x.activation {
                        1 => Activation::Sigmoid,
                        2 => Activation::Tahn,
                        3 => Activation::Softmax,
                        4 => Activation::LeakyRelu(x.value),
                        5 => Activation::ExpRelu(x.value),
                        6 => Activation::Linear(x.value),
                        _ => panic!()
                    }
                })
                .collect::<Vec<_>>()
        }
    }
}

