
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
pub struct GRU {
    pub input_size: u32,
    pub memory_size: u32,
    pub output_size: u32,
    pub current_memory: Vec<f32>,
    pub current_output: Vec<f32>,
    pub gate_forget: Dense,
    pub gate_output: Dense,
    pub gate_extract: Dense,
}



/// implement a simple GRU layer that can be added to the neat network
impl GRU {


    pub fn new(input_size: u32, memory_size: u32, output_size: u32) -> Self {        
        let network_in_size = input_size + memory_size + output_size;
        GRU {
            input_size,
            memory_size,
            output_size,
            current_memory: vec![0.0; memory_size as usize],
            current_output: vec![0.0; output_size as usize],
            gate_forget: Dense::new(network_in_size, memory_size, LayerType::DensePool, Activation::Sigmoid),
            gate_output: Dense::new(network_in_size, output_size, LayerType::DensePool, Activation::Tahn),
            gate_extract: Dense::new(network_in_size, memory_size, LayerType::DensePool, Activation::Tahn),
        }
    }


}




/// implement the layer trait for the GRU so it can be stored in the neat network
impl Layer for GRU {


    /// implement the propagation function for the GRU layer 
    #[inline]
    fn forward(&mut self, inputs: &Vec<f32>, trace: bool) -> Option<Vec<f32>> {
        let mut concat_input_output = self.current_output.clone();
        concat_input_output.extend(inputs);

        let mut network_input = concat_input_output.clone();
        network_input.extend(&self.current_memory);

        // calculate memory updates
        let mut forget = self.gate_forget.forward(&network_input, trace)?;
        let mut memory = self.gate_extract.forward(&network_input, trace)?;

        // figure out what to forget from the current memory
        vectorops::element_multiply(&mut self.current_memory, &forget);
        vectorops::element_invert(&mut forget);
        vectorops::element_multiply(&mut memory, &forget);
        vectorops::element_add(&mut self.current_memory, &memory);

        // add the new memory for the input to the output network of the layer
        concat_input_output.extend(&self.current_memory);

        // calculate the current output of the layer
        self.current_output = self.gate_output.forward(&concat_input_output, trace)?;
        Some(self.current_output.clone())
    }


    fn backward(&mut self, errors: &Vec<f32>, learning_rate: f32, update: bool) -> Option<Vec<f32>> {
        let output_error = self.gate_output.backward(&errors, learning_rate, update)?;
        // let delta_mem = self.current_memory
        //     .iter()
        //     .zip(output_error.iter())
        //     .map(|(a, b)| {
        //         a * b
        //     })
        //     .collect::<Vec<_>>();

        // let delta_out = errors
        //     .iter()
        //     .zip(output_error.iter())
        //     .map(|(a, b)| {
        //         a * b
        //     })
        //     .collect::<Vec<_>>();
        
        self.gate_forget.backward(&output_error, learning_rate, update)?;
        self.gate_extract.backward(&output_error, learning_rate, update)?;

        
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
        (self.input_size as usize, self.output_size as usize)
    }

}


/// Implement clone for the neat neural network in order to facilitate 
/// proper crossover and mutation for the network
impl Clone for GRU {

    #[inline]
    fn clone(&self) -> Self {
        GRU {
            input_size: self.input_size,
            memory_size: self.memory_size,
            output_size: self.output_size,
            current_memory: vec![0.0; self.memory_size as usize],
            current_output: vec![0.0; self.output_size as usize],
            gate_forget: self.gate_forget.clone(),
            gate_output: self.gate_output.clone(),
            gate_extract: self.gate_extract.clone(),
        }
    }
}




/// in order for the GRU layer to be evolved along with the rest of the network, Genome must be implemented 
/// so that the layer can be crossed over and measured along with other GRU layers 
impl Genome<GRU, NeatEnvironment> for GRU
    where GRU: Layer
{

    /// implement how to crossover two GRU layers 
    #[inline]
    fn crossover(child: &GRU, parent_two: &GRU, env: &Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<GRU> {
        let child = GRU {
            input_size: child.input_size,
            memory_size: child.memory_size,
            output_size: child.output_size,
            current_memory: vec![0.0; child.memory_size as usize],
            current_output: vec![0.0; child.output_size as usize],
            gate_forget: Dense::crossover(&child.gate_forget, &parent_two.gate_forget, env, crossover_rate)?,
            gate_output: Dense::crossover(&child.gate_output, &parent_two.gate_output, env, crossover_rate)?,
            gate_extract: Dense::crossover(&child.gate_extract, &parent_two.gate_extract, env, crossover_rate)?,
        };
        Some(child)
    }


    /// get the distance between two GRU layers of the network
    #[inline]
    fn distance(one: &GRU, two: &GRU, env: &Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut result = 0.0;
        result += Dense::distance(&one.gate_forget, &two.gate_forget, env);
        result += Dense::distance(&one.gate_output, &two.gate_output, env);
        result += Dense::distance(&one.gate_extract, &two.gate_extract, env);
        result
    }
}

/// These must be implemneted for the network or any type to be 
/// used within seperate threads. Because implementing the functions 
/// themselves is dangerious and unsafe and i'm not smart enough 
/// to do that from scratch, these "implmenetaions" will get rid 
/// of the error and realistically they don't need to be implemneted for the
/// program to work
unsafe impl Send for GRU {}
unsafe impl Sync for GRU {}
/// implement display for the GRU layer of the network
impl fmt::Display for GRU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let address: u64 = mem::transmute(self);
            write!(f, "GRU=[{}]", address)
        }
    }
}
