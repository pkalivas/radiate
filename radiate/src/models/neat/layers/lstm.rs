
extern crate rand;

use std::mem;
use std::fmt;
use std::any::Any;
use super::{
    layertype::LayerType,
    layer::Layer,
    dense::Dense,
};
use super::super::activation::Activation;

use crate::Genome;



#[derive(Debug)]
pub struct LSTM {
    pub input_size: i32,
    pub output_size: i32,
    pub hidden_states: Vec<Dense>,
    pub hidden_cells: Vec<Dense>,
    pub forget_gate: Dense,
    pub input_gate: Dense,
    pub gated_gate: Dense,
    pub output_gate: Dense
}



impl LSTM {

    pub fn new(input_size: i32, layer_size: i32, output_size: i32) -> Self {
        LSTM {
            input_size: input_size,
            output_size: output_size,
            hidden_states: Vec::new(),
            hidden_cells: Vec::new(),
            forget_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Sigmoid),
            input_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Sigmoid),
            gated_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Tahn),
            output_gate: Dense::new(input_size * 2, layer_size, LayerType::Dense, Activation::Sigmoid)
        }
    }

}




impl Layer for LSTM {

    fn propagate(&mut self, inputs: &Vec<f64>) -> Option<Vec<f64>> {
        
        None
    }

    fn backprop(&mut self, errors: &Vec<f64>, learning_rate: f64) -> Option<Vec<f64>> {

        None
    }


    fn as_ref_any(&self) -> &dyn Any
        where Self: Sized + 'static
    {
        self
    }


    fn as_mut_any(&mut self) -> &mut dyn Any
        where Self: Sized + 'static
    {
        self
    }

    fn shape(&self) -> (usize, usize) {
        (self.input_size as usize, self.output_size as usize)
    }

}


/// Implement clone for the neat neural network in order to facilitate 
/// proper crossover and mutation for the network
impl Clone for LSTM {
    fn clone(&self) -> Self {
        LSTM {
            input_size: self.input_size,
            output_size: self.output_size,
            hidden_states: self.hidden_states
                .iter()
                .map(|x| x.clone())
                .collect(),
            hidden_cells: self.hidden_cells
                .iter()
                .map(|x| x.clone())
                .collect(),
            forget_gate: self.forget_gate.clone(),
            input_gate: self.input_gate.clone(),
            gated_gate: self.gated_gate.clone(),
            output_gate: self.output_gate.clone()
        }

    }
}

/// These must be implemneted for the network or any type to be 
/// used within seperate threads. Because implementing the functions 
/// themselves is dangerious and unsafe and i'm not smart enough 
/// to do that from scratch, these "implmenetaions" will get rid 
/// of the error and realistically they don't need to be implemneted for the
/// program to work
unsafe impl Send for LSTM {}
unsafe impl Sync for LSTM {}
/// implement display for the lstm layer of the network
impl fmt::Display for LSTM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let address: u64 = mem::transmute(self);
            write!(f, "LSTM=[{}]", address)
        }
    }
}
