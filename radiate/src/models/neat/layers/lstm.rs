
extern crate rand;

use std::fmt;
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::thread;
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




/// LSTM State is meant to be a 'snapshot' of the outputs for each
/// gate at each time step. The rest of the time-step memories are held in tracers
#[derive(Debug, Serialize, Deserialize)]
pub struct LSTMState {
    pub f_gate_output: Vec<Vec<f32>>,
    pub i_gate_output: Vec<Vec<f32>>,
    pub s_gate_output: Vec<Vec<f32>>,
    pub o_gate_output: Vec<Vec<f32>>,
    pub memory_states: Vec<Vec<f32>>,
    pub d_prev_memory: Option<Vec<f32>>,
    pub d_prev_hidden: Option<Vec<f32>>
}



impl LSTMState {


    pub fn new() -> Self {
        LSTMState {
            f_gate_output: Vec::new(),
            i_gate_output: Vec::new(),
            s_gate_output: Vec::new(),
            o_gate_output: Vec::new(),
            memory_states: Vec::new(),
            d_prev_memory: None,
            d_prev_hidden: None
        }
    }


    /// add the gate outputs to the state for this time step
    pub fn update_forward(&mut self, fg: Vec<f32>, ig: Vec<f32>, sg: Vec<f32>, og: Vec<f32>, mem_state: Vec<f32>) {
        self.f_gate_output.push(fg);
        self.i_gate_output.push(ig);
        self.s_gate_output.push(sg);
        self.o_gate_output.push(og);
        self.memory_states.push(mem_state);
    }
}




/// LSTM is a long-short term memory cell represented by a collection of Dense layers and two
/// distinct memory vectors which get updated and travel 'through time'
#[derive(Debug, Serialize, Deserialize)]
pub struct LSTM {
    pub input_size: u32,
    pub memory_size: u32,
    pub output_size: u32,
    pub activation: Activation,
    pub memory: Vec<f32>,
    pub hidden: Vec<f32>,
    pub states: LSTMState,
    pub g_gate: Arc<RwLock<Dense>>,
    pub i_gate: Arc<RwLock<Dense>>,
    pub f_gate: Arc<RwLock<Dense>>,
    pub o_gate: Arc<RwLock<Dense>>,
    pub v_gate: Arc<RwLock<Dense>>
}



impl LSTM {


    pub fn new(input_size: u32, memory_size: u32, output_size: u32, activation: Activation) -> Self {
        let cell_input = input_size + memory_size;
        LSTM {
            input_size,
            memory_size,
            output_size,
            activation,
            memory: vec![0.0; memory_size as usize],
            hidden: vec![0.0; memory_size as usize],
            states: LSTMState::new(),
            g_gate: Arc::new(RwLock::new(Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Tanh))),
            i_gate: Arc::new(RwLock::new(Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid))),
            f_gate: Arc::new(RwLock::new(Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid))),
            o_gate: Arc::new(RwLock::new(Dense::new(cell_input, memory_size, LayerType::DensePool, Activation::Sigmoid))),
            v_gate: Arc::new(RwLock::new(Dense::new(memory_size, output_size, LayerType::DensePool, activation)))
        }
    }



    /// Feed forward with each forward propagation being executed in a separate thread to speed up
    /// the forward pass if the network is NOT being evolved. If it is, there are already so many threads
    /// working to optimize the entire population that extra threading is unnecessary and might actually slow it down
    #[inline]
    pub fn step_forward_async(&mut self, inputs: &[f32]) -> Option<Vec<f32>> {
        // get the previous state and output and create the input to the layer
        let mut hidden_input = self.hidden.clone();
        hidden_input.extend(inputs);

        // clone all the gates to prevent lifetime conflicts
        let g_gate_clone = Arc::clone(&self.g_gate);
        let o_gate_clone = Arc::clone(&self.o_gate);
        let f_gate_clone = Arc::clone(&self.f_gate);
        let i_gate_clone = Arc::clone(&self.i_gate);

        // get all the gate outputs 
        let hidden_async = Arc::new(hidden_input);
        let g_input = Arc::clone(&hidden_async);
        let o_input = Arc::clone(&hidden_async);
        let f_input = Arc::clone(&hidden_async);
        let i_input = Arc::clone(&hidden_async);

        // spawn the threads 
        let g_output = thread::spawn(move || { return g_gate_clone.write().unwrap().forward(&*g_input).unwrap(); });
        let o_output = thread::spawn(move || { return o_gate_clone.write().unwrap().forward(&*o_input).unwrap(); });
        let f_output = thread::spawn(move || { return f_gate_clone.write().unwrap().forward(&*f_input).unwrap(); });
        let i_output = thread::spawn(move || { return i_gate_clone.write().unwrap().forward(&*i_input).unwrap(); });

        // current memory and output need to be mutable but we also want to save that data for bptt
        let mut curr_state = g_output.join().ok()?;
        let mut curr_output = o_output.join().ok()?;
        let f_curr = f_output.join().ok()?;
        let i_curr = i_output.join().ok()?;

        let g_out = curr_state.clone();
        let o_out = curr_output.clone();

        // update the current state 
        vectorops::element_multiply(&mut self.memory, &f_curr);
        vectorops::element_multiply(&mut curr_state, &i_curr);
        vectorops::element_add(&mut self.memory, &curr_state);
        vectorops::element_multiply(&mut curr_output, &vectorops::element_activate(&self.memory, Activation::Tanh));

        // update the state parameters only if the gates are traceable and the data needs to be collected
        self.states.update_forward(f_curr, i_curr, g_out, o_out, self.memory.clone());   
        
        // return the output of the layer
        // keep track of the memory and the current output and the current state
        self.hidden = curr_output;
        self.v_gate.write().unwrap().forward(&self.hidden)
    }



    /// step forward synchronously
    #[inline]
    pub fn step_forward(&mut self, inputs: &[f32]) -> Option<Vec<f32>> {
        // get the previous state and output and create the input to the layer
        // let mut previous_state = &mut self.memory;
        let mut hidden_input = self.hidden.clone();
        hidden_input.extend(inputs);

        // get all the gate outputs 
        let f_output = self.f_gate.write().unwrap().forward(&hidden_input)?;
        let i_output = self.i_gate.write().unwrap().forward(&hidden_input)?;
        let o_output = self.o_gate.write().unwrap().forward(&hidden_input)?;
        let g_output = self.g_gate.write().unwrap().forward(&hidden_input)?;

        // current memory and output need to be mutable but we also want to save that data for bptt
        let mut current_state = g_output.clone();
        let mut current_output = o_output.clone();

        // update the current state 
        vectorops::element_multiply(&mut self.memory, &f_output);
        vectorops::element_multiply(&mut current_state, &i_output);
        vectorops::element_add(&mut self.memory, &current_state);
        vectorops::element_multiply(&mut current_output, &vectorops::element_activate(&self.memory, Activation::Tanh));

        // return the output of the layer
        // keep track of the memory and the current output and the current state
        self.hidden = current_output;
        self.v_gate.write().unwrap().forward(&self.hidden)
    }



    /// Preform one step backwards for the layer. Set the tracer historical meta data to look at the current
    /// index, and use that data to compute the gradient steps for each weight in each gated network.
    /// If update is true, the gates will take the accumulated gradient steps, and add them to their respective weight values
    #[inline]
    pub fn step_back(&mut self, errors: &Vec<f32>, l_rate: f32) -> Option<Vec<f32>> {
        // get the derivative of the cell and hidden state from the previous step as well as the previous memory state
        let dh_next = self.states.d_prev_hidden.clone()?;
        let dc_next = self.states.d_prev_memory.clone()?;

        // unpack the current gate outputs 
        let c_old = self.states.memory_states.pop()?;
        let g_curr = self.states.s_gate_output.pop()?;
        let i_curr = self.states.i_gate_output.pop()?;
        let f_curr = self.states.f_gate_output.pop()?;
        let o_curr = self.states.o_gate_output.pop()?;

        
        // compute the hidden to output gradient
        // dh = error @ Wy.T + dh_next
        let mut dh = self.v_gate.write().unwrap().backward(errors, l_rate)?;
        vectorops::element_add(&mut dh, &dh_next);

        // Gradient for ho in h = ho * tanh(c)     
        //dho = tanh(c) * dh
        //dho = dsigmoid(ho) * dho
        let mut dho = vectorops::element_activate(&c_old, Activation::Tanh);
        vectorops::element_multiply(&mut dho, &dh);
        vectorops::element_multiply(&mut dho, &vectorops::element_deactivate(&o_curr, self.o_gate.read().unwrap().activation));
        let o_gate_clone = Arc::clone(&self.o_gate);
        let o_handle = thread::spawn(move || { 
            return o_gate_clone.write().unwrap().backward(&dho, l_rate).unwrap(); 
        });
        
        // Gradient for c in h = ho * tanh(c), note we're adding dc_next here     
        // dc = ho * dh * dtanh(c)
        // dc = dc + dc_next
        let mut dc = vectorops::product(&o_curr, &dh);
        vectorops::element_multiply(&mut dc, &vectorops::element_deactivate(&c_old, Activation::Tanh));
        vectorops::element_add(&mut dc, &dc_next);

        // Gradient for hf in c = hf * c_old + hi * hc    
        // dhf = c_old * dc
        // dhf = dsigmoid(hf) * dhf
        let mut dhf = vectorops::product(&c_old, &dc);
        vectorops::element_multiply(&mut dhf, &vectorops::element_deactivate(&f_curr, self.f_gate.read().unwrap().activation));
        let f_gate_clone = Arc::clone(&self.f_gate);
        let f_handle = thread::spawn(move || { 
            return f_gate_clone.write().unwrap().backward(&dhf, l_rate).unwrap(); 
        });

        // Gradient for hi in c = hf * c_old + hi * hc     
        // dhi = hc * dc
        // dhi = dsigmoid(hi) * dhi
        let mut dhi = vectorops::product(&g_curr, &dc);
        vectorops::element_multiply(&mut dhi, &vectorops::element_deactivate(&i_curr, self.i_gate.read().unwrap().activation));
        let i_gate_clone = Arc::clone(&self.i_gate);
        let i_handle = thread::spawn(move || { 
            return i_gate_clone.write().unwrap().backward(&dhi, l_rate).unwrap(); 
        });

        // Gradient for hc in c = hf * c_old + hi * hc     
        // dhc = hi * dc
        // dhc = dtanh(hc) * dhc
        let mut dhc = vectorops::product(&i_curr, &dc);
        vectorops::element_multiply(&mut dhc, &vectorops::element_deactivate(&g_curr, self.g_gate.read().unwrap().activation));
        let g_gate_clone = Arc::clone(&self.g_gate);
        let g_handle = thread::spawn(move || { 
            return g_gate_clone.write().unwrap().backward(&dhc, l_rate).unwrap(); 
        });

        // As X was used in multiple gates, the gradient must be accumulated here     
        // dX = dXo + dXc + dXi + dXf
        let mut dx = vec![0.0; (self.input_size + self.memory_size) as usize];
        vectorops::element_add(&mut dx, &o_handle.join().ok()?);
        vectorops::element_add(&mut dx, &f_handle.join().ok()?);
        vectorops::element_add(&mut dx, &i_handle.join().ok()?);
        vectorops::element_add(&mut dx, &g_handle.join().ok()?);
        
        // Split the concatenated X, so that we get our gradient of h_old     
        // dh_next = dx[:, :H]
        let dh_next = dx[..self.memory_size as usize].to_vec();
        let dc_next = vectorops::product(&f_curr, &dc);
        
        // Gradient for c_old in c = hf * c_old + hi * hc     
        // dc_next = hf * dc
        self.states.d_prev_hidden = Some(dh_next);
        self.states.d_prev_memory = Some(dc_next);

        // return the error of the input given to the layer
        Some(dx[..self.input_size as usize].to_vec())
    }

}



#[typetag::serde]
impl Layer for LSTM {


    /// forward propagate inputs, if the model is being evolved don't spawn extra threads because
    /// it slows down the process by about double the original time. If the model is being trained
    /// traditionally, step forward asynchronously by spawning a thread for each individual gate
    /// which results in speeds about double as a synchronous thread.
    #[inline]
    fn forward(&mut self, inputs: &Vec<f32>) -> Option<Vec<f32>> {
        if self.f_gate.read().map(|x| x.trace_states.is_some()).ok()? {
            return self.step_forward_async(inputs);
        }
        self.step_forward(inputs)
    }



    /// apply backpropagation through time asynchronously because this is not done during evolution
    #[inline]
    fn backward(&mut self, errors: &Vec<f32>, learning_rate: f32) -> Option<Vec<f32>> {
        if self.states.d_prev_hidden.is_none() && self.states.d_prev_memory.is_none() {
            self.states.d_prev_memory = Some(vec![0.0; self.memory_size as usize]);      
            self.states.d_prev_hidden = Some(vec![0.0; self.memory_size as usize]);          
        }

        // preform the step back for this iteration
        self.step_back(errors, learning_rate)
    }



    /// reset the lstm network by clearing the tracer and the states as well as the memory and hidden state
    fn reset(&mut self) {
        self.g_gate.write().unwrap().reset();
        self.i_gate.write().unwrap().reset();
        self.f_gate.write().unwrap().reset();
        self.o_gate.write().unwrap().reset();
        self.v_gate.write().unwrap().reset();
        self.states = LSTMState::new();
        self.memory = vec![0.0; self.memory_size as usize];
        self.hidden = vec![0.0; self.memory_size as usize];
    }



    /// add tracers to all the gate.write().unwrap()s in the layer 
    fn add_tracer(&mut self) {
        self.g_gate.write().unwrap().add_tracer();
        self.i_gate.write().unwrap().add_tracer();
        self.f_gate.write().unwrap().add_tracer();
        self.o_gate.write().unwrap().add_tracer();
        self.v_gate.write().unwrap().add_tracer();
    }


    /// remove the tracers from all the gate.write().unwrap()s in the layer
    fn remove_tracer(&mut self) {
        self.g_gate.write().unwrap().remove_tracer();
        self.i_gate.write().unwrap().remove_tracer();
        self.f_gate.write().unwrap().remove_tracer();
        self.o_gate.write().unwrap().remove_tracer();
        self.v_gate.write().unwrap().remove_tracer();
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
/// proper crossover and mutation for the network  (*self.environment.read().unwrap()).clone();
impl Clone for LSTM {

    #[inline]
    fn clone(&self) -> Self {
        LSTM {
            input_size: self.input_size,
            memory_size: self.memory_size,
            output_size: self.output_size,
            activation: self.activation.clone(),
            memory: vec![0.0; self.memory_size as usize],
            hidden: vec![0.0; self.memory_size as usize],
            states: LSTMState::new(),
            g_gate: Arc::new(RwLock::new((*self.g_gate.read().unwrap()).clone())), 
            i_gate: Arc::new(RwLock::new((*self.i_gate.read().unwrap()).clone())), 
            f_gate: Arc::new(RwLock::new((*self.f_gate.read().unwrap()).clone())), 
            o_gate: Arc::new(RwLock::new((*self.o_gate.read().unwrap()).clone())),
            v_gate: Arc::new(RwLock::new((*self.v_gate.read().unwrap()).clone()))
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
    fn crossover(child: &LSTM, parent_two: &LSTM, env: Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<LSTM> {
        let child = LSTM {
            input_size: child.input_size,
            memory_size: child.memory_size,
            output_size: child.output_size,
            activation: child.activation,
            memory: vec![0.0; child.memory_size as usize],
            hidden: vec![0.0; child.memory_size as usize],
            states: LSTMState::new(),
            g_gate: Arc::new(RwLock::new(Dense::crossover(&child.g_gate.read().unwrap(), &parent_two.g_gate.read().unwrap(), Arc::clone(&env), crossover_rate)?)),
            i_gate: Arc::new(RwLock::new(Dense::crossover(&child.i_gate.read().unwrap(), &parent_two.i_gate.read().unwrap(), Arc::clone(&env), crossover_rate)?)),
            f_gate: Arc::new(RwLock::new(Dense::crossover(&child.f_gate.read().unwrap(), &parent_two.f_gate.read().unwrap(), Arc::clone(&env), crossover_rate)?)),
            o_gate: Arc::new(RwLock::new(Dense::crossover(&child.o_gate.read().unwrap(), &parent_two.o_gate.read().unwrap(), Arc::clone(&env), crossover_rate)?)),
            v_gate: Arc::new(RwLock::new(Dense::crossover(&child.v_gate.read().unwrap(), &parent_two.v_gate.read().unwrap(), Arc::clone(&env), crossover_rate)?)),
        };
        Some(child)
    }


    /// get the distance between two LSTM layers of the network
    #[inline]
    fn distance(one: &LSTM, two: &LSTM, env: Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut result = 0.0;
        result += Dense::distance(&one.g_gate.read().unwrap(), &two.g_gate.read().unwrap(), Arc::clone(&env));
        result += Dense::distance(&one.i_gate.read().unwrap(), &two.i_gate.read().unwrap(), Arc::clone(&env));
        result += Dense::distance(&one.f_gate.read().unwrap(), &two.f_gate.read().unwrap(), Arc::clone(&env));
        result += Dense::distance(&one.o_gate.read().unwrap(), &two.o_gate.read().unwrap(), Arc::clone(&env));
        result += Dense::distance(&one.v_gate.read().unwrap(), &two.v_gate.read().unwrap(), Arc::clone(&env));
        result
    }
}

/// implement display for the LSTM layer of the network
impl fmt::Display for LSTM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LSTM=[input={}, memory={}, output={}]",
          self.input_size, self.memory_size, self.output_size)
    }
}
