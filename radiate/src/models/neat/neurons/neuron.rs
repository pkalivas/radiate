

use std::fmt::{
    Debug,
    Formatter,
    Result
};
use std::collections::HashMap;


pub trait Neuron: NeuronClone {

    fn reset(&mut self);

    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64;

    fn deactivate(&mut self, curr_value: f64) -> f64; 

}


pub trait NeuronClone {
    fn clone_box(&self) -> Box<dyn Neuron>;
}


impl<N> NeuronClone for N 
    where N: 'static + Neuron + Clone 
{
    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new((*self).clone())
    }
}



impl Debug for dyn Neuron {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Neuron")
    }
}


impl Clone for Box<dyn Neuron> {
    fn clone(&self) -> Box<dyn Neuron> {
        self.clone_box()
    }
}