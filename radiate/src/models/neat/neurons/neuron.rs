

use std::collections::HashMap;
use std::fmt::{
    Debug,
    Formatter,
    Result
};


/// Neuron is what is held within a vertex. This is meant to 
/// represent a different node on the NEAT network graph.
/// Because NEAT needs to be cloned, so does whatever impelemnts this trait and 
/// therefore so does this. 
pub trait Neuron: NeuronClone {
    /// After each feed forward of the neat graph, the neuron needs to be reset
    /// otherwise the answers being pushed out of the network will be wrong or potentially 
    /// carry over answers that are incorrect. Pluse different types of nodes might need 
    /// to carry information over throughout feedforwards. ie: LSTM nodes need to keep track
    /// of cell states, recurrent nodes need past states, ect.
    fn reset(&mut self);
    /// Each neuron also needs a method of activation. A dense neuron (simple feed forward layer) simply 
    /// has to sum the inputs and put that total through an activation function, but an LSTM neuron or 
    /// recurrent neurons have much different methods of activation
    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64;
    /// Much like deactivation, the mulitutde of vairables in different types of neurons 
    /// leads to very different types of deactivation (gradient computation) for backpropagating
    /// through the network. Because of this, for the NEAT network to work correctly with 
    /// different types of neurons within it, each type of neuron must define it's own way to deactivate
    fn deactivate(&mut self, curr_value: f64) -> f64; 
}


/// Turns out cloning an unsized boxed trait object is much more complicated than
/// i originally thought it would be. Because of this, this workaround is needed. 
/// A neuron clone trait is only to be implemented within this scope for Neuron, 
/// all it does is expose a function for a generic 'N' to be cloned
pub trait NeuronClone {
    /// Define a function which takes a self (meant to be a dyn Neuron) and 
    /// returns a clone of the implementing object as a boxed dyn neuron
    fn clone_box(&self) -> Box<dyn Neuron>;
}


/// NeuronClone needs to be implemented over N which is meant to be
/// a dyn Neyron, which returns a boxed version of self
impl<N> NeuronClone for N 
    where N: 'static + Neuron + Clone 
{
    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new((*self).clone())
    }
}


/// In order to debug NEAT, debug needs to be implemented for all the structs
/// within the object. Dyn traits are not 'debuggable', so the function must be implemented 
impl Debug for dyn Neuron {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Neuron")
    }
}


/// Now that we have NeuronClone which is implemeneted for all N
/// which implemented Clone and Neuron, we can clone any Box<dyn neuron>
/// usuing the function clone_box for NeuronClone.
impl Clone for Box<dyn Neuron> {
    fn clone(&self) -> Box<dyn Neuron> {
        self.clone_box()
    }
}