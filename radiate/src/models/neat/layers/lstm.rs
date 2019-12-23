
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
    pub hidden_state: Vec<f64>,
    pub memory_cell: Vec<f64>,
    pub memory_gate: Dense,
    pub forget_gate: Dense,
    pub input_gate: Dense,
    pub gated_gate: Dense,
    pub output_gate: Dense,
    pub output_layer: Dense
}



impl LSTM {

    pub fn new(input_size: i32, layer_size: i32, output_size: i32) -> Self {
        LSTM {
            input_size: input_size,
            output_size: output_size,
            hidden_state: (0..layer_size)
                .into_iter()
                .map(|_| 0.0)
                .collect(),
            memory_cell: (0..layer_size)
                .into_iter()
                .map(|_| 0.0)
                .collect(),
            memory_gate: Dense::new(layer_size, layer_size, LayerType::Dense, Activation::Tahn),
            forget_gate: Dense::new(input_size + layer_size, layer_size, LayerType::Dense, Activation::Sigmoid),
            input_gate: Dense::new(input_size + layer_size, layer_size, LayerType::Dense, Activation::Sigmoid),
            gated_gate: Dense::new(input_size + layer_size, layer_size, LayerType::Dense, Activation::Tahn),
            output_gate: Dense::new(input_size + layer_size, layer_size, LayerType::Dense, Activation::Sigmoid),
            output_layer: Dense::new(layer_size, output_size, LayerType::Dense, Activation::Sigmoid)
        }
    }

}




impl Layer for LSTM {

    fn propagate(&mut self, inputs: &Vec<f64>) -> Option<Vec<f64>> {
        self.hidden_state.extend(inputs); 


        println!("Forget ");
        let forget_gate_result = self.forget_gate.propagate(&self.hidden_state)?;
        println!("input");
        let input_gate_result = self.input_gate.propagate(&self.hidden_state)?;
        println!("gated");
        let gated_gate_result = self.gated_gate.propagate(&self.hidden_state)?;
        println!("output");
        let output_gate_result = self.output_gate.propagate(&self.hidden_state)?;

        let harrmond_input_gated = input_gate_result
            .iter()
            .zip(gated_gate_result.iter())
            .map(|(i, g)| i * g)
            .collect::<Vec<_>>();
        let new_memory = forget_gate_result
            .iter()
            .zip(self.memory_cell.iter())
            .enumerate()
            .map(|(index, (f, m))| (f * m) + harrmond_input_gated[index])
            .collect::<Vec<_>>();

        println!("new mem: {:?}", new_memory);
        let memory_gate_result = self.memory_gate.propagate(&new_memory)?;
            println!("mem");
        let new_hidden_state = output_gate_result
            .iter()
            .zip(memory_gate_result.iter())
            .map(|(h, m)| h * m)
            .collect::<Vec<_>>();
        
        self.memory_cell = new_memory;
        self.hidden_state = new_hidden_state;
        println!("output");
        let output = self.output_layer.propagate(&self.hidden_state)?;
        
        Some(output)
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
            hidden_state: self.hidden_state
                .iter()
                .map(|x| x.clone())
                .collect(),
            memory_cell: self.memory_cell
                .iter()
                .map(|x| x.clone())
                .collect(),
            memory_gate: self.memory_gate.clone(),
            forget_gate: self.forget_gate.clone(),
            input_gate: self.input_gate.clone(),
            gated_gate: self.gated_gate.clone(),
            output_gate: self.output_gate.clone(),
            output_layer: self.output_layer.clone()
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
