
extern crate rand;

use super::{
    layertype::LayerType,
    layer::Layer,
    dense::Dense,
};
use super::super::activation::Activation;

use crate::Genome;



pub struct LSTM {
    pub hidden_states: Vec<Dense>,
    pub hidden_cells: Vec<Dense>,
    pub forget_gate: Dense,
    pub input_gate: Dense,
    pub gated_gate: Dense,
    pub output_gate: Dense
}


impl LSTM {

    pub fn new(input_size: i32, layer_size: i32) -> Self {
        LSTM {
            hidden_states: Vec::new(),
            hidden_cells: Vec::new(),
            forget_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Sigmoid),
            input_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Sigmoid),
            gated_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Tahn),
            output_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Sigmoid)
        }
    }

}