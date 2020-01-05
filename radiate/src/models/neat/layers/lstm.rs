
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


// https://wiseodd.github.io/techblog/2016/08/12/lstm-backprop/


#[derive(Debug)]
pub struct LSTMState {
    pub size: usize,
    pub concat_input: Vec<Vec<f32>>,
    pub f_gate_output: Vec<Vec<f32>>,
    pub i_gate_output: Vec<Vec<f32>>,
    pub s_gate_output: Vec<Vec<f32>>,
    pub o_gate_output: Vec<Vec<f32>>,
    pub memory_states: Vec<Vec<f32>>,
    pub hidden_states: Vec<Vec<f32>>,
    pub outputs: Vec<Vec<f32>>,
    pub errors: Vec<Vec<f32>>,
    pub d_prev_memory: Vec<Vec<f32>>,
    pub d_prev_hidden: Vec<Vec<f32>>
}


impl LSTMState {

    pub fn new() -> Self {
        LSTMState {
            size: 0,
            concat_input: Vec::new(),
            f_gate_output: Vec::new(),
            i_gate_output: Vec::new(),
            s_gate_output: Vec::new(),
            o_gate_output: Vec::new(),
            memory_states: Vec::new(),
            hidden_states: Vec::new(),
            outputs: Vec::new(),
            errors: Vec::new(),
            d_prev_memory: Vec::new(),
            d_prev_hidden: Vec::new()
        }
    }

    pub fn update_forward(&mut self, c_input: Vec<f32>, fg: Vec<f32>, ig: Vec<f32>, sg: Vec<f32>, og: Vec<f32>, mem_state: Vec<f32>, out_state: Vec<f32>, output: Vec<f32>) {
        self.concat_input.push(c_input);
        self.f_gate_output.push(fg);
        self.i_gate_output.push(ig);
        self.s_gate_output.push(sg);
        self.o_gate_output.push(og);
        self.memory_states.push(mem_state);
        self.hidden_states.push(out_state);
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
    pub hidden: Vec<f32>,
    pub states: LSTMState,
    pub g_gate: Dense,
    pub i_gate: Dense,
    pub f_gate: Dense,
    pub o_gate: Dense,
    pub v_gate: Dense
}


impl LSTM {

    pub fn new(input_size: u32, memory_size: u32, output_size: u32) -> Self {
        let cell_input = input_size + memory_size;
        LSTM {
            input_size,
            memory_size,
            memory: vec![0.0; memory_size as usize],
            hidden: vec![0.0; memory_size as usize],
            states: LSTMState::new(),
            g_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Tahn),
            i_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            f_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            o_gate: Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid),
            v_gate: Dense::new(memory_size, output_size, LayerType::Dense, Activation::Sigmoid)
        }
    }


    fn set_trace_index(&mut self, index: usize) {
        self.g_gate.set_trace(index);
        self.i_gate.set_trace(index);
        self.f_gate.set_trace(index);
        self.o_gate.set_trace(index);
        self.v_gate.set_trace(index);
    }


    fn reset_traces(&mut self) {
        self.g_gate.trace_states.reset();
        self.i_gate.trace_states.reset();
        self.f_gate.trace_states.reset();
        self.o_gate.trace_states.reset();
        self.v_gate.trace_states.reset();
        self.states = LSTMState::new();
    }


    pub fn step_back(&mut self, l_rate: f32, trace: bool, update: bool, index: usize) -> Option<Vec<f32>> {
        // get the previous memory and the current error
        self.set_trace_index(index);
        let dh_next = self.states.d_prev_hidden.last()?;
        let dc_next = self.states.d_prev_memory.last()?;
        let c_old = if index as i32 - 1 < 0 {
            vec![0.0_f32; self.memory_size as usize]
        } else {
            self.states.memory_states.get(index)?.clone()
        };


        // compute the hidden to output gradient
        // dWy = h.T @ dy
        // let dWy = vectorops::product(self.states.hidden_states.get(index)?, self.states.errors.get(index)?);
        // self.v_gate.backward(self.states.errors.get(index)?, l_rate, trace, update);

        let mut dh = self.v_gate.backward(self.states.errors.get(index)?, l_rate, trace, update)?;
        vectorops::element_multiply(&mut dh, &dh_next);


        // Gradient for ho in h = ho * tanh(c)     
        //dho = tanh(c) * dh
        //dho = dsigmoid(ho) * dho
        let mut dho = vectorops::element_activate(self.states.memory_states.get(index)?, Activation::Tahn);
        vectorops::element_multiply(&mut dho, &dh);
        vectorops::element_multiply(&mut dho, &vectorops::element_deactivate(self.states.o_gate_output.get(index)?, self.o_gate.activation));
        

        // Gradient for c in h = ho * tanh(c), note we're adding dc_next here     
        // dc = ho * dh * dtanh(c)
        // dc = dc + dc_next
        let mut dc = vectorops::product(self.states.o_gate_output.get(index)?, &dh);
        vectorops::element_multiply(&mut dc, &vectorops::element_deactivate(self.states.memory_states.get(index)?, Activation::Tahn));
        vectorops::element_add(&mut dc, &dc_next);


        // Gradient for hf in c = hf * c_old + hi * hc    
        // dhf = c_old * dc
        // dhf = dsigmoid(hf) * dhf
        let mut dhf = vectorops::product(&c_old, &dc);
        vectorops::element_multiply(&mut dhf, &vectorops::element_deactivate(self.states.f_gate_output.get(index)?, self.f_gate.activation));


        // Gradient for hi in c = hf * c_old + hi * hc     
        // dhi = hc * dc
        // dhi = dsigmoid(hi) * dhi
        let mut dhi = vectorops::product(self.states.s_gate_output.get(index)?, &dc);
        vectorops::element_multiply(&mut dhi, &vectorops::element_deactivate(self.states.i_gate_output.get(index)?, self.i_gate.activation));


        // Gradient for hc in c = hf * c_old + hi * hc     
        // dhc = hi * dc
        // dhc = dtanh(hc) * dhc
        let mut dhc = vectorops::product(self.states.i_gate_output.get(index)?, &dc);
        vectorops::element_multiply(&mut dhc, &vectorops::element_deactivate(self.states.s_gate_output.get(index)?, self.g_gate.activation));

        let f_error = self.f_gate.backward(&dhf, l_rate, trace, update)?;
        let i_error = self.i_gate.backward(&dhi, l_rate, trace, update)?;
        let g_error = self.g_gate.backward(&dhc, l_rate, trace, update)?;
        let o_error = self.o_gate.backward(&dho, l_rate, trace, update)?;

        // As X was used in multiple gates, the gradient must be accumulated here     
        //      dX = dXo + dXc + dXi + dXf
        // Split the concatenated X, so that we get our gradient of h_old     
        //      dh_next = dX[:, :H]
        // Gradient for c_old in c = hf * c_old + hi * hc     
        //      dc_next = hf * dc

        // add up the error of the input from the gates
        let mut dX = vec![0.0; (self.input_size + self.memory_size) as usize];
        vectorops::element_add(&mut dX, &f_error);
        vectorops::element_add(&mut dX, &i_error);
        vectorops::element_add(&mut dX, &g_error);
        vectorops::element_add(&mut dX, &o_error);

        let dh_next = dX[..self.memory_size as usize].to_vec();
        let dc_next = vectorops::product(self.states.f_gate_output.get(index)?, &dc);

        self.states.d_prev_hidden.push(dh_next);
        self.states.d_prev_memory.push(dc_next);

        Some(dX[self.memory_size as usize..].to_vec())
    }


}



impl Layer for LSTM {


    #[inline]
    fn forward(&mut self, inputs: &Vec<f32>, trace: bool) -> Option<Vec<f32>> {
        // get the previous state and output and create the input to the layer
        // let mut previous_state = &mut self.memory;
        let mut hidden_input = self.hidden.clone();
        hidden_input.extend(inputs);

        // get all the gate outputs 
        let f_output = self.f_gate.forward(&hidden_input, trace)?;
        let i_output = self.i_gate.forward(&hidden_input, trace)?;
        let o_output = self.o_gate.forward(&hidden_input, trace)?;
        let g_output = self.g_gate.forward(&hidden_input, trace)?;

        // current memory and output need to be mutable but we also want to save that data for bptt
        let mut current_state = g_output.clone();
        let mut current_output = o_output.clone();

        // update the current state 
        vectorops::element_multiply(&mut self.memory, &f_output);
        vectorops::element_multiply(&mut current_state, &i_output);
        vectorops::element_add(&mut self.memory, &current_state);
        vectorops::element_multiply(&mut current_output, &vectorops::element_activate(&self.memory, Activation::Tahn));

        // compute the output of the layer
        let layer_out = self.v_gate.forward(&current_output, trace)?;
        
        // update the state parameters - can this be sped up?
        self.states.update_forward(hidden_input, f_output, i_output, g_output, o_output, self.memory.clone(), current_output.clone(), layer_out.clone());
        
        // keep track of the memory and the current output and the current state
        self.hidden = current_output;

        // return the output of the layer
        Some(layer_out)
    }


    /// apply backpropagation through time 
    #[inline]
    fn backward(&mut self, errors: &Vec<f32>, learning_rate: f32, trace: bool, update: bool) -> Option<Vec<f32>> {
        // regardless of if the network needs to be updated, the error needs to be stored
        self.states.update_backward(errors.clone());

        // backpropagation throught time if update is true
        if update {
            // need next states as well, but the first iteration they will be 0
            self.states.d_prev_memory.push(vec![0.0; self.memory_size as usize]);      
            self.states.d_prev_hidden.push(vec![0.0; self.memory_size as usize]);          

            // leave room for one last backward step to update all the weights and step backward once
            for i in (1..self.states.size).rev() {
                self.step_back(learning_rate, trace, false, i);
            }

            let result = self.step_back(learning_rate, trace, true, 0);
            self.reset_traces();
            return Some(result?);
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
            hidden: vec![0.0; self.memory_size as usize],
            states: LSTMState::new(),
            g_gate: self.g_gate.clone(), 
            i_gate: self.i_gate.clone(), 
            f_gate: self.f_gate.clone(), 
            o_gate: self.o_gate.clone(),
            v_gate: self.v_gate.clone()
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
            hidden: vec![0.0; child.memory_size as usize],
            states: LSTMState::new(),
            g_gate: Dense::crossover(&child.g_gate, &parent_two.g_gate, env, crossover_rate)?,
            i_gate: Dense::crossover(&child.i_gate, &parent_two.i_gate, env, crossover_rate)?,
            f_gate: Dense::crossover(&child.f_gate, &parent_two.f_gate, env, crossover_rate)?,
            o_gate: Dense::crossover(&child.o_gate, &parent_two.o_gate, env, crossover_rate)?,
            v_gate: Dense::crossover(&child.v_gate, &parent_two.v_gate, env, crossover_rate)?
        };
        Some(child)
    }


    /// get the distance between two LSTM layers of the network
    #[inline]
    fn distance(one: &LSTM, two: &LSTM, env: &Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut result = 0.0;
        result += Dense::distance(&one.g_gate, &two.g_gate, env);
        result += Dense::distance(&one.i_gate, &two.i_gate, env);
        result += Dense::distance(&one.f_gate, &two.f_gate, env);
        result += Dense::distance(&one.o_gate, &two.o_gate, env);
        // result += Dense::distance(&one.v_gate, &two.v_gate, env);
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
