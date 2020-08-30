
extern crate rand;

use std::fmt;
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




#[derive(Debug, Serialize, Deserialize)]
pub struct GRU {
    pub input_size: u32,
    pub memory_size: u32,
    pub output_size: u32,
    pub current_memory: Vec<f32>,
    pub current_output: Vec<f32>,
    pub f_gate: Dense,
    pub e_gate: Dense,
    pub o_gate: Dense,
}



/// implement a simple GRU layer that can be added to the neat network
impl GRU {


    pub fn new(input_size: u32, memory_size: u32, output_size: u32, act: Activation) -> Self {        
        let network_in_size = input_size + memory_size + output_size;
        GRU {
            input_size,
            memory_size,
            output_size,
            current_memory: vec![0.0; memory_size as usize],
            current_output: vec![0.0; output_size as usize],
            f_gate: Dense::new(network_in_size, memory_size, LayerType::DensePool, Activation::Sigmoid),
            e_gate: Dense::new(network_in_size, memory_size, LayerType::DensePool, Activation::Tanh),
            o_gate: Dense::new(network_in_size, output_size, LayerType::DensePool, act),
        }
    }


}




/// implement the layer trait for the GRU so it can be stored in the neat network
#[typetag::serde]
impl Layer for GRU {


    /// implement the propagation function for the GRU layer 
    #[inline]
    fn forward(&mut self, inputs: &Vec<f32>) -> Option<Vec<f32>> {
        let mut concat_input_output = self.current_output.clone();
        concat_input_output.extend(inputs);

        let mut network_input = concat_input_output.clone();
        network_input.extend(&self.current_memory);

        // calculate memory updates
        let mut forget = self.f_gate.forward(&network_input)?;
        let mut memory = self.e_gate.forward(&network_input)?;

        // figure out what to forget from the current memory
        vectorops::element_multiply(&mut self.current_memory, &forget);
        vectorops::element_invert(&mut forget);
        vectorops::element_multiply(&mut memory, &forget);
        vectorops::element_add(&mut self.current_memory, &memory);

        // add the new memory for the input to the output network of the layer
        concat_input_output.extend(&self.current_memory);

        // calculate the current output of the layer
        self.current_output = self.o_gate.forward(&concat_input_output)?;
        Some(self.current_output.clone())
    }


    fn backward(&mut self, _errors: &Vec<f32>, _learning_rate: f32) -> Option<Vec<f32>> {
        panic!("Backprop for GRU is not implemented yet");
        // let output_error = self.o_gate.backward(&errors, learning_rate)?;
        // // let delta_mem = self.current_memory
        // //     .iter()
        // //     .zip(output_error.iter())
        // //     .map(|(a, b)| {
        // //         a * b
        // //     })
        // //     .collect::<Vec<_>>();

        // // let delta_out = errors
        // //     .iter()
        // //     .zip(output_error.iter())
        // //     .map(|(a, b)| {
        // //         a * b
        // //     })
        // //     .collect::<Vec<_>>();
        
        // self.f_gate.backward(&output_error, learning_rate)?;
        // self.e_gate.backward(&output_error, learning_rate)?;

        
        // Some(errors.clone())
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
            f_gate: self.f_gate.clone(),
            o_gate: self.o_gate.clone(),
            e_gate: self.e_gate.clone(),
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
    fn crossover(child: &GRU, parent_two: &GRU, env: Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<GRU> {
        let child = GRU {
            input_size: child.input_size,
            memory_size: child.memory_size,
            output_size: child.output_size,
            current_memory: vec![0.0; child.memory_size as usize],
            current_output: vec![0.0; child.output_size as usize],
            f_gate: Dense::crossover(&child.f_gate, &parent_two.f_gate, Arc::clone(&env), crossover_rate)?,
            o_gate: Dense::crossover(&child.o_gate, &parent_two.o_gate, Arc::clone(&env), crossover_rate)?,
            e_gate: Dense::crossover(&child.e_gate, &parent_two.e_gate, Arc::clone(&env), crossover_rate)?,
        };
        Some(child)
    }


    /// get the distance between two GRU layers of the network
    #[inline]
    fn distance(one: &GRU, two: &GRU, env: Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut result = 0.0;
        result += Dense::distance(&one.f_gate, &two.f_gate, Arc::clone(&env));
        result += Dense::distance(&one.o_gate, &two.o_gate, Arc::clone(&env));
        result += Dense::distance(&one.e_gate, &two.e_gate, Arc::clone(&env));
        result
    }
}

/// implement display for the GRU layer of the network
impl fmt::Display for GRU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GRU=[input={}, memory={}, output={}]",
          self.input_size, self.memory_size, self.output_size)
    }
}
