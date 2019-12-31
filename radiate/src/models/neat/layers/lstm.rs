
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
    pub cell_size: u32,
    pub output_size: u32,
    pub memory: Vec<Vec<f32>>,
    pub output: Vec<Vec<f32>>,
    pub activation_gate: Dense,
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
            activation_gate: Dense::new(cell_input, cell_size, LayerType::DensePool, Activation::Tahn),
            state_gate: Dense::new(cell_size, output_size, LayerType::DensePool, Activation::Tahn),
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

        let mut activation_output = self.activation_gate.forward(&previous_output)?;
        let input_output = self.input_gate.forward(&previous_output)?;

        vectorops::element_multiply(&mut activation_output, &input_output);
        vectorops::element_add(&mut previous_state, &activation_output);

        let mut output_output = self.output_gate.forward(&previous_output)?;
        let state_output = self.state_gate.forward(&previous_state)?;

        vectorops::element_multiply(&mut output_output, &state_output);

        self.memory.push(previous_state.clone());
        self.output.push(output_output.clone());

        Some(output_output)
    }


    fn backward(&mut self, errors: &Vec<f32>, learning_rate: f32, update_weights: bool) -> Option<Vec<f32>> {
        
        // simple backpropagation without respect to the state and forget output at t+1
        let mut previous_state = self.memory.last()?.clone();
        let previous_previous_state = self.memory.get(self.memory.len() - 2)?;
        let mut state_derivative = self.output_gate.get_outputs()?;

        // state derivative
        let state_gate_error = self.state_gate.backward(&previous_state, learning_rate, update_weights)?;
        vectorops::element_multiply(&mut state_derivative, &errors);
        vectorops::element_multiply(&mut state_derivative, &state_gate_error);

        // activation gate derivative
        let mut activation_gate_derivative = self.input_gate.get_outputs()?;
        let mut activation_gate_output = self.activation_gate.get_outputs()?;
        activation_gate_output.iter_mut().for_each(|x| *x = *x * *x);
        vectorops::element_multiply(&mut activation_gate_derivative, &state_derivative);
        vectorops::element_invert(&mut activation_gate_output);
        vectorops::element_multiply(&mut activation_gate_derivative, &activation_gate_output);

        // input gate derivative
        let mut input_gate_derivative = self.activation_gate.get_outputs()?;
        let mut input_gate_output = self.input_gate.get_outputs()?;
        vectorops::element_multiply(&mut input_gate_derivative, &state_derivative);
        vectorops::element_multiply(&mut input_gate_derivative, &input_gate_output);
        vectorops::element_invert(&mut input_gate_output);
        vectorops::element_multiply(&mut input_gate_derivative, &input_gate_output);

        // forget gate derivative
        let mut forget_gate_derivative = self.forget_gate.get_outputs()?;
        vectorops::element_invert(&mut forget_gate_derivative);
        vectorops::element_multiply(&mut forget_gate_derivative, &previous_previous_state);
        vectorops::element_multiply(&mut forget_gate_derivative, &previous_state);
        vectorops::element_multiply(&mut forget_gate_derivative, &self.forget_gate.get_outputs()?);

        // output gate derivative 
        let mut output_gate_derivative = self.output_gate.get_outputs()?;
        vectorops::element_invert(&mut output_gate_derivative);
        vectorops::element_multiply(&mut output_gate_derivative, &errors);
        vectorops::element_multiply(&mut output_gate_derivative, &self.output_gate.get_outputs()?);
        vectorops::element_squeeze(&mut previous_state, Activation::Tahn);
        vectorops::element_multiply(&mut output_gate_derivative, &previous_state);

        // i think this is wrong from here down, need to finsih by calculating the input error and multipying it 
        // by the error of the input found here
        // : https://medium.com/@aidangomez/let-s-do-this-f9b699de31d9
        let mut activation_error = self.activation_gate.backward(&activation_gate_derivative, learning_rate, update_weights)?;
        let input_error = self.input_gate.backward(&input_gate_derivative, learning_rate, update_weights)?;
        let forget_error = self.forget_gate.backward(&forget_gate_derivative, learning_rate, update_weights)?;
        let output_error = self.output_gate.backward(&output_gate_derivative, learning_rate, update_weights)?;

        vectorops::element_multiply(&mut activation_error, &input_error);
        vectorops::element_multiply(&mut activation_error, &forget_error);
        vectorops::element_multiply(&mut activation_error, &output_error);
        
        Some(activation_error)
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
            activation_gate: self.activation_gate.clone(),
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
            activation_gate: Dense::crossover(&child.activation_gate, &parent_two.activation_gate, env, crossover_rate)?,
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
        result += Dense::distance(&one.activation_gate, &two.activation_gate, env);
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
