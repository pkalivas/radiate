

use std::collections::HashMap;
use std::fmt::{
    Debug,
    Formatter,
    Result
};


/// Neuron is meant to encapsulate specific neuron logic. Meaning a feed forward
/// neuron is much different than a recurrent or LSTM or CTRNN. Because of this,
/// separating that logic out and allowing it to be implemented independant of the 
/// Vertex is needed for proper design
pub trait Neuron: NeuronClone {
 
    fn reset(&mut self);
 
    /// Each neuron also needs a method of activation. A dense neuron (simple feed forward layer) simply 
    /// has to sum the inputs and put that total through an activation function, but an LSTM neuron or 
    /// recurrent neurons have much different methods of activation
    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64;
 
    /// Much like deactivation, the mulitutde of vairables in different types of neurons 
    fn deactivate(&mut self, curr_value: f64) -> f64; 
}


/// Turns out cloning an unsized boxed trait object is much more complicated than
/// i originally thought it would be. Because of this, this workaround is needed. 
/// A neuron clone trait is only to be implemented within this scope for Neuron, 
/// all it does is expose a function for a generic 'N' to be cloned
pub trait NeuronClone {
 
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



/// Now that we have NeuronClone which is implemeneted for all N
/// which implemented Clone and Neuron, we can clone any Box<dyn neuron>
impl Clone for Box<dyn Neuron> {
    fn clone(&self) -> Box<dyn Neuron> {
        self.clone_box()
    }
}



impl Debug for dyn Neuron {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Neuron")
    }
}
