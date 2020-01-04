
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
pub struct LSTMState {
    pub size: usize,
    pub concat_input: Vec<Vec<f32>>,
    pub f_gate_output: Vec<Vec<f32>>,
    pub i_gate_output: Vec<Vec<f32>>,
    pub s_gate_output: Vec<Vec<f32>>,
    pub o_gate_output: Vec<Vec<f32>>,
    pub memory_states: Vec<Vec<f32>>,
    pub output_states: Vec<Vec<f32>>,
    pub outputs: Vec<Vec<f32>>,
    pub errors: Vec<Vec<f32>>
}


impl LSTMState {

    pub fn new(memory_size: u32) -> Self {
        LSTMState {
            size: 1,
            concat_input: Vec::new(),
            f_gate_output: Vec::new(),
            i_gate_output: Vec::new(),
            s_gate_output: Vec::new(),
            o_gate_output: Vec::new(),
            memory_states: vec![vec![0.0; memory_size as usize]],
            output_states: vec![vec![0.0; memory_size as usize]],
            outputs: Vec::new(),
            errors: Vec::new()
        }
    }

    pub fn update_forward(&mut self, c_input: Vec<f32>, fg: Vec<f32>, ig: Vec<f32>, sg: Vec<f32>, og: Vec<f32>, mem_state: Vec<f32>, out_state: Vec<f32>, output: Vec<f32>) {
        self.concat_input.push(c_input);
        self.f_gate_output.push(fg);
        self.i_gate_output.push(ig);
        self.s_gate_output.push(sg);
        self.o_gate_output.push(og);
        self.memory_states.push(mem_state);
        self.output_states.push(out_state);
        self.outputs.push(output);
        self.size += 1;
    }

    pub fn update_backward(&mut self, errors: Vec<f32>) {
        self.errors.push(errors);
    }



}




#[derive(Debug)]
pub struct LSTM {
    pub input_size: u32,
    pub memory_size: u32,
    pub memory: Vec<f32>,
    pub output: Vec<f32>,
    pub lstm_state: LSTMState,
    pub state_gate: Dense,
    pub input_gate: Dense,
    pub forget_gate: Dense,
    pub output_gate: Dense,
    pub hidden_out: Dense
}


impl LSTM {

    pub fn new(input_size: u32, memory_size: u32, output_size: u32) -> Self {
        let cell_input = input_size + memory_size;
        LSTM {
            input_size,
            memory_size,
            memory: vec![0.0; memory_size as usize],
            output: vec![0.0; memory_size as usize],
            lstm_state: LSTMState::new(memory_size),
            state_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Tahn),
            input_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            forget_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            output_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            hidden_out: Dense::new(memory_size, output_size, LayerType::Dense, Activation::Sigmoid)
        }
    }


    fn set_trace_index(&mut self, index: usize) {
        // println!("SETTING INDEX: {:?}", index);
        self.state_gate.set_trace(index);
        self.input_gate.set_trace(index);
        self.forget_gate.set_trace(index);
        self.output_gate.set_trace(index);
        self.hidden_out.set_trace(index);
    }


    fn reset_traces(&mut self) {
        self.state_gate.trace_states.reset();
        self.input_gate.trace_states.reset();
        self.forget_gate.trace_states.reset();
        self.output_gate.trace_states.reset();
        self.hidden_out.trace_states.reset();
    }


    pub fn step_back(&mut self, l_rate: f32, trace: bool, update: bool, index: usize) -> Option<Vec<f32>> {
        // get the previous memory and the current error
        self.set_trace_index(index);
        let curr_memory = self.lstm_state.memory_states.get(index)?;
        let curr_error = self.lstm_state.errors.get(index)?;


        //-------- compute the derivative of the hidden layer to output and feed it backward to get hidden layer error -------//
        // derivative of the hidden to output gate of the layer.
        let mut h_out_error = self.hidden_out.backward(&curr_error, l_rate, trace, update)?;
        vectorops::element_add(&mut h_out_error, self.lstm_state.output_states.get(index + 1)?);
        

        //-------- compute the derivative of the output gate and feed it backward to get the output gate error --------//
        // output derivative 
        let mut d_o_gate = vectorops::element_deactivate(&curr_memory, Activation::Tahn);
        vectorops::element_multiply(&mut d_o_gate, &h_out_error);
        let o_gate_direction = vectorops::element_deactivate(self.lstm_state.o_gate_output.get(index)?, self.output_gate.activation);
        vectorops::element_multiply(&mut d_o_gate, &o_gate_direction);
        let o_gate_error = self.output_gate.backward(&d_o_gate, l_rate, trace, update)?;


        //-------- compute the derivative of the memory and state gate --------//
        // memory and state gate derivate and error
        let mut d_memory = h_out_error.clone();
        let memory_error = self.lstm_state.memory_states.get(index + 1)?.clone();
        let act_memory = vectorops::element_activate(&curr_memory, Activation::Tahn);
        vectorops::element_multiply(&mut d_memory, self.lstm_state.o_gate_output.get(index)?);
        vectorops::element_multiply(&mut d_memory, &vectorops::element_deactivate(&act_memory, Activation::Tahn));
        vectorops::element_add(&mut d_memory, &memory_error);

        let mut d_s_gate = self.lstm_state.i_gate_output.get(index)?.clone();
        let act_state = vectorops::element_deactivate(self.lstm_state.s_gate_output.get(index)?, self.state_gate.activation);
        vectorops::element_multiply(&mut d_s_gate, &d_memory);
        vectorops::element_multiply(&mut d_s_gate, &act_state);
        let s_gate_error = self.state_gate.backward(&d_s_gate, l_rate, trace, update)?;


        //-------- compute the derivative of the input gate --------//
        let mut d_i_gate = self.lstm_state.s_gate_output.get(index)?.clone();
        let act_input = vectorops::element_deactivate(self.lstm_state.i_gate_output.get(index)?, self.input_gate.activation);
        vectorops::element_multiply(&mut d_i_gate, &d_memory);
        vectorops::element_multiply(&mut d_i_gate, &act_input);
        let i_gate_error = self.input_gate.backward(&d_i_gate, l_rate, trace, update)?;


        //-------- compute the derivative of the forget gate --------//
        let mut d_f_gate = self.lstm_state.memory_states.get(index - 1)?.clone();
        let act_forget = vectorops::element_activate(self.lstm_state.f_gate_output.get(index)?, self.input_gate.activation);
        vectorops::element_multiply(&mut d_f_gate, &d_memory);
        vectorops::element_multiply(&mut d_f_gate, &act_forget);
        let f_gate_error = self.forget_gate.backward(&d_f_gate, l_rate, trace, update)?;


        //-------- compute the error of th entire layer --------//
        let mut step_error = vec![0.0; self.input_size as usize + self.memory_size as usize];
        vectorops::element_add(&mut step_error, &o_gate_error);
        vectorops::element_add(&mut step_error, &s_gate_error);
        vectorops::element_add(&mut step_error, &i_gate_error);
        vectorops::element_add(&mut step_error, &f_gate_error);

        // get the result 
        let out = step_error[self.memory_size as usize..].to_vec();
        Some(out)
    }


}



impl Layer for LSTM {


    #[inline]
    fn forward(&mut self, inputs: &Vec<f32>, trace: bool) -> Option<Vec<f32>> {
        // get the previous state and output and create the input to the layer
        let mut previous_state = &mut self.memory;
        let mut previous_output = self.output.clone();
        previous_output.extend(inputs);

        // get all the gate outputs 
        let forget_output = self.forget_gate.forward(&previous_output, trace)?;
        let state_output = self.state_gate.forward(&previous_output, trace)?;
        let input_output = self.input_gate.forward(&previous_output, trace)?;
        let output_output = self.output_gate.forward(&previous_output, trace)?;

        // current memory and output need to be mutable but we also want to save that data for bptt
        let mut current_state = state_output.clone();
        let mut current_output = output_output.clone();

        // update the current state 
        vectorops::element_multiply(&mut previous_state, &forget_output);
        vectorops::element_multiply(&mut current_state, &input_output);
        vectorops::element_add(&mut previous_state, &current_state);
        vectorops::element_multiply(&mut current_output, &vectorops::element_activate(&previous_state, Activation::Tahn));

        // compute the output of the layer
        let layer_out = self.hidden_out.forward(&current_output, trace)?;
        
        // update the state parameters - can this be sped up?
        self.lstm_state.update_forward(previous_output, forget_output, input_output, state_output, output_output, previous_state.clone(), current_output.clone(), layer_out.clone());
        
        // keep track of the memory and the current output and the current state
        self.output = current_output;

        // return the output of the layer
        Some(layer_out)
    }


    /// apply backpropagation through time 
    #[inline]
    fn backward(&mut self, errors: &Vec<f32>, learning_rate: f32, trace: bool, update: bool) -> Option<Vec<f32>> {
        // regardless of if the network needs to be updated, the error needs to be stored
        self.lstm_state.update_backward(errors.clone());

        // backpropagation throught time if update is true
        if update {
            // need next states as well, but the first iteration they will be 0
            self.lstm_state.output_states.push(vec![0.0; self.memory_size as usize]);
            self.lstm_state.memory_states.push(vec![0.0; self.memory_size as usize]);

            // leave room for one last backward step to update all the weights and step backward once
            for i in (2..self.lstm_state.size).rev() {
                // println!("{:?}", i);
                self.step_back(learning_rate, trace, false, i);
            }
            let result = self.step_back(learning_rate, trace, true, 1);
            // println!("{:#?}", self);
            self.lstm_state = LSTMState::new(self.memory_size);
            self.reset_traces();
            return result
        }
        Some(errors.clone())
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
            memory: vec![0.0; self.memory_size as usize],
            output: vec![0.0; self.memory_size as usize],
            lstm_state: LSTMState::new(self.memory_size),
            state_gate: self.state_gate.clone(), 
            input_gate: self.input_gate.clone(), 
            forget_gate: self.forget_gate.clone(), 
            output_gate: self.output_gate.clone(),
            hidden_out: self.hidden_out.clone()
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
            memory: vec![0.0; child.memory_size as usize],
            output: vec![0.0; child.memory_size as usize],
            lstm_state: LSTMState::new(child.memory_size),
            state_gate: Dense::crossover(&child.state_gate, &parent_two.state_gate, env, crossover_rate)?,
            input_gate: Dense::crossover(&child.input_gate, &parent_two.input_gate, env, crossover_rate)?,
            forget_gate: Dense::crossover(&child.forget_gate, &parent_two.forget_gate, env, crossover_rate)?,
            output_gate: Dense::crossover(&child.output_gate, &parent_two.output_gate, env, crossover_rate)?,
            hidden_out: Dense::crossover(&child.hidden_out, &parent_two.hidden_out, env, crossover_rate)?
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
        // result += Dense::distance(&one.hidden_out, &two.hidden_out, env);
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
