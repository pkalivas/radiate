
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


#[derive(Debug, Clone)]
pub struct LSTMState {
    pub memory: Vec<Vec<f32>>,
    pub output: Vec<Vec<f32>>,
    pub input: Vec<Vec<f32>>,
    pub state_output: Vec<Vec<f32>>,
    pub input_output: Vec<Vec<f32>>,
    pub forget_output: Vec<Vec<f32>>,
    pub output_output: Vec<Vec<f32>>
}


#[derive(Debug, Clone)]
pub struct StateDifference {
    pub memory_error: Vec<Vec<f32>>,
    pub output_error: Vec<Vec<f32>>,
    pub given_error: Vec<Vec<f32>>,
    pub given_input: Vec<Vec<f32>>,
}

impl StateDifference {
    pub fn new(memory_size: u32, output_size: u32, input_size: u32) -> Self {
        StateDifference {
            memory_error: vec![vec![0.0; memory_size as usize]],
            output_error: vec![vec![0.0; output_size as usize]],
            given_error: vec![vec![0.0; output_size as usize]],
            given_input: vec![vec![0.0; input_size as usize]]
        }
    }
}



#[derive(Debug)]
pub struct LSTM {
    pub input_size: u32,
    pub cell_size: u32,
    pub output_size: u32,
    pub memory: Vec<Vec<f32>>,
    pub output: Vec<Vec<f32>>,
    pub state_errors: StateDifference,
    pub state_gate: Dense,
    pub input_gate: Dense,
    pub forget_gate: Dense,
    pub output_gate: Dense
}


impl LSTM {

    pub fn new(input_size: u32, cell_size: u32, output_size: u32) -> Self {
        let cell_input = input_size + output_size;
        LSTM {
            input_size,
            cell_size,
            output_size,
            memory: vec![vec![0.0; cell_size as usize]],
            output: vec![vec![0.0; output_size as usize]],
            state_errors: StateDifference::new(cell_size, output_size, input_size),
            state_gate: Dense::new(cell_input, cell_size, LayerType::DensePool, Activation::Tahn),
            input_gate: Dense::new(cell_input, cell_size, LayerType::DensePool, Activation::Sigmoid),
            forget_gate: Dense::new(cell_input, cell_size, LayerType::DensePool, Activation::Sigmoid),
            output_gate: Dense::new(cell_input, output_size, LayerType::DensePool, Activation::Sigmoid)
        }
    }

}



impl Layer for LSTM {

    fn forward(&mut self, inputs: &Vec<f32>) -> Option<Vec<f32>> {
        let mut previous_state = self.memory.last()?.clone();
        let mut previous_output = self.output.last()?.clone();
        previous_output.extend(inputs);

        let forget_output = self.forget_gate.forward(&previous_output)?;

        vectorops::element_multiply(&mut previous_state, &forget_output);

        let mut activation_output = self.state_gate.forward(&previous_output)?;
        let input_output = self.input_gate.forward(&previous_output)?;

        vectorops::element_multiply(&mut activation_output, &input_output);
        vectorops::element_add(&mut previous_state, &activation_output);

        let mut output_output = self.output_gate.forward(&previous_output)?;
        let state_output = vectorops::element_activate(&previous_state, Activation::Tahn); // is there a shape mismatch here?

        vectorops::element_multiply(&mut output_output, &state_output);

        self.memory.push(previous_state.clone());
        self.output.push(output_output.clone());

        Some(output_output)
    }


    fn backward(&mut self, errors: &Vec<f32>, learning_rate: f32, update_weights: bool) -> Option<Vec<f32>> {
        
        // push the current errors to the state_errors struct to keep track for bptt
        self.state_errors.given_error.push(errors.clone());

        // get the most recent hidden state, output, and their previous errors 
        let mut previous_output_derivative = self.state_errors.output_error.last()?.clone();
        let previous_memory_derivative = self.state_errors.memory_error.last()?;
        vectorops::element_add(&mut previous_output_derivative, &errors);

        // get the most recent outputs 
        let mut previous_memory = self.memory.last()?.clone();
        let mut input_gate_output = self.input_gate.get_outputs()?.clone();
        // let mut state_gate_output = self.state_gate.get_outputs()?.clone();
        let mut output_gate_output = self.output_gate.get_outputs()?.clone();
        let mut forget_gate_output = self.forget_gate.get_outputs()?.clone();
        let mut state_gate_output = self.state_gate.get_outputs()?.clone();
        
        // memory derivative in output_gate_output
        vectorops::element_multiply(&mut output_gate_output, &previous_memory_derivative);
        vectorops::element_add(&mut output_gate_output, &previous_output_derivative);
        
        // output derivative
        vectorops::element_multiply(&mut previous_memory, &previous_output_derivative);
        
        // input gate derivative 
        vectorops::element_multiply(&mut state_gate_output, &output_gate_output);
        
        // activation gate derivative
        vectorops::element_multiply(&mut input_gate_output, &output_gate_output);
        
        // forget gate derivative
        vectorops::element_multiply(&mut previous_memory, &output_gate_output);
        
        
        // compute the gate errors 
        let mut input_gate_error = vectorops::element_deactivate(&self.input_gate.get_outputs()?, self.input_gate.activation);
        vectorops::element_multiply(&mut input_gate_error, &state_gate_output);
        
        let mut forget_gate_error = vectorops::element_deactivate(&self.forget_gate.get_outputs()?, self.forget_gate.activation);
        vectorops::element_multiply(&mut forget_gate_error, &previous_memory);
        
        let mut output_gate_error = vectorops::element_deactivate(&self.output_gate.get_outputs()?, self.output_gate.activation);
        vectorops::element_multiply(&mut output_gate_error, &previous_memory);
        
        let mut state_gate_error = vectorops::element_deactivate(&self.state_gate.get_outputs()?, self.state_gate.activation);
        vectorops::element_multiply(&mut state_gate_error, &input_gate_output);
        
        
        // update the weights
        let input = self.input_gate.backward(&input_gate_error, learning_rate, update_weights)?;
        let forget = self.forget_gate.backward(&forget_gate_error, learning_rate, update_weights)?;
        let output = self.output_gate.backward(&output_gate_error, learning_rate, update_weights)?;
        let activation = self.state_gate.backward(&state_gate_error, learning_rate, update_weights)?;
        // println!("here");

        let mut layer_error = vec![0.0; self.input_size as usize];
        vectorops::element_add(&mut layer_error, &input);
        vectorops::element_add(&mut layer_error, &forget);
        vectorops::element_add(&mut layer_error, &output);
        vectorops::element_add(&mut layer_error, &activation);

        // save the erros from this time step
        let error = layer_error[self.input_size as usize..].to_vec();
        if update_weights {
            self.state_errors = StateDifference::new(self.cell_size, self.output_size, self.input_size);
        } else {
            vectorops::element_multiply(&mut forget_gate_output, &output_gate_output);
            self.state_errors.memory_error.push(previous_memory);
            self.state_errors.output_error.push(error.clone());
        }
        Some(error)
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

    #[inline]
    fn clone(&self) -> Self {
        LSTM {
            input_size: self.input_size,
            cell_size: self.cell_size,
            output_size: self.output_size,
            memory: vec![vec![0.0; self.cell_size as usize]],
            output: vec![vec![0.0; self.output_size as usize]],
            state_errors: StateDifference::new(self.cell_size, self.output_size, self.input_size),
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
            cell_size: child.cell_size,
            output_size: child.output_size,
            memory: vec![vec![0.0; child.cell_size as usize]],
            output: vec![vec![0.0; child.output_size as usize]],
            state_errors: StateDifference::new(child.cell_size, child.output_size, child.input_size),
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
