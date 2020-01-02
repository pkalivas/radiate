
extern crate rand;

use std::fmt;
use std::mem;
use std::any::Any;
use std::sync::{Arc, RwLock};
use super::{
    layertype::LayerType,
    layer::Layer,
    dense::Dense,
    vectorops
};    
use super::super::{
    activation::Activation,
    neatenv::NeatEnvironment,
};    

use crate::Genome;




#[derive(Debug)]
pub struct LSTM {
    pub input_size: u32,
    pub memory_size: u32,
    pub memory: Vec<Vec<f32>>,
    pub output: Vec<Vec<f32>>,
    pub state_gate: Dense,
    pub input_gate: Dense,
    pub forget_gate: Dense,
    pub output_gate: Dense
}


impl LSTM {

    pub fn new(input_size: u32, memory_size: u32) -> Self {
        let cell_input = input_size + memory_size;
        LSTM {
            input_size,
            memory_size,
            memory: vec![vec![0.0; memory_size as usize]],
            output: vec![vec![0.0; memory_size as usize]],
            state_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Tahn),
            input_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            forget_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            output_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid)
        }
    }

}



impl Layer for LSTM {


    #[inline]
    fn forward(&mut self, inputs: &Vec<f32>) -> Option<Vec<f32>> {
        // get the previous state and output and create the input to the layer
        let mut previous_state = self.memory.last()?.clone();
        let mut previous_output = self.output.last()?.clone();
        previous_output.extend(inputs);

        // get all the gate outputs 
        let forget_output = self.forget_gate.forward(&previous_output)?;
        let state_output = self.state_gate.forward(&previous_output)?;
        let input_output = self.input_gate.forward(&previous_output)?;
        let output_output = self.output_gate.forward(&previous_output)?;

        // current memory and output need to be mutable but we also want to save that data for bptt
        let mut current_state = state_output.clone();
        let mut current_output = output_output.clone();

        // update the current state 
        vectorops::element_multiply(&mut previous_state, &forget_output);
        vectorops::element_multiply(&mut current_state, &input_output);
        vectorops::element_add(&mut previous_state, &current_state);
        vectorops::element_multiply(&mut current_output, &vectorops::element_activate(&previous_state, Activation::Tahn));

        // keep track of the memory and the current output and the current state
        self.memory.push(previous_state.clone());
        self.output.push(current_output.clone());

        // return the output of the layer
        Some(current_output)
    }


    /// apply backpropagation through time 
    #[inline]
    fn backward(&mut self, errors: &Vec<f32>, learning_rate: f32, update: bool) -> Option<Vec<f32>> {
        // first iteration is coming directly from the output layer so the inputs are simple 
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
        (self.input_size as usize, self.memory_size as usize)
    }
}


/// Implement clone for the neat neural network in order to facilitate 
/// proper crossover and mutation for the network
impl Clone for LSTM {

    #[inline]
    fn clone(&self) -> Self {
        LSTM {
            input_size: self.input_size,
            memory_size: self.memory_size,
            memory: vec![vec![0.0; self.memory_size as usize]],
            output: vec![vec![0.0; self.memory_size as usize]],
            state_gate: self.state_gate.clone(), 
            input_gate: self.input_gate.clone(), 
            forget_gate: self.forget_gate.clone(), 
            output_gate: self.output_gate.clone()
        }
    }
}




/// in order for the lstm layer to be evolved along with the rest of the network, Genome must be implemented 
/// so that the layer can be crossed over and measured along with other lstm layers 
impl Genome<LSTM, NeatEnvironment> for LSTM
    where LSTM: Layer
{

    /// implement how to crossover two LSTM layers 
    #[inline]
    fn crossover(child: &LSTM, parent_two: &LSTM, env: &Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<LSTM> {
        let child = LSTM {
            input_size: child.input_size,
            memory_size: child.memory_size,
            memory: vec![vec![0.0; child.memory_size as usize]],
            output: vec![vec![0.0; child.memory_size as usize]],
            state_gate: Dense::crossover(&child.state_gate, &parent_two.state_gate, env, crossover_rate)?,
            input_gate: Dense::crossover(&child.input_gate, &parent_two.input_gate, env, crossover_rate)?,
            forget_gate: Dense::crossover(&child.forget_gate, &parent_two.forget_gate, env, crossover_rate)?,
            output_gate: Dense::crossover(&child.output_gate, &parent_two.output_gate, env, crossover_rate)?,
        };
        Some(child)
    }


    /// get the distance between two LSTM layers of the network
    #[inline]
    fn distance(one: &LSTM, two: &LSTM, env: &Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut result = 0.0;
        result += Dense::distance(&one.state_gate, &two.state_gate, env);
        result += Dense::distance(&one.input_gate, &two.input_gate, env);
        result += Dense::distance(&one.forget_gate, &two.forget_gate, env);
        result += Dense::distance(&one.output_gate, &two.output_gate, env);
        result
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
/// implement display for the LSTM layer of the network
impl fmt::Display for LSTM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let address: u64 = mem::transmute(self);
            write!(f, "LSTM=[{}]", address)
        }
    }
}
