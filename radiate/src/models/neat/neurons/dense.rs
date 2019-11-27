

use std::collections::HashMap;
use super::Activation;
use super::neuron::Neuron;


/// A dense neuron, meaning a basic feed forward neural network
/// neuron. Impleming Neuron means it can be used in the network
#[derive(Debug, Clone)]
pub struct Dense {
    pub activation: Activation,
}


impl Dense {

    /// This is a pretty simple neuron and all it's logic is encapsulated
    /// in it's neuron implementation, so all it needs to know is it's 
    /// activation function
    pub fn new(activation: Activation) -> Self {
        Dense { activation }
    }

}



impl Neuron for Dense {

    /// Dense doesn't need to reset anything outside of 
    /// a vertex's reset function so this can be empty
    fn reset(&mut self) { }

    /// activate a dense neuron by summing it's inputs and squashing 
    /// that total through the dense neuron's activation function 
    /// If one of the inputs is None (it shouldn't be), panic! because this 
    /// neuron should not be activated
    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64 {
        let total = incoming.values()
            .fold(0.0, |sum, curr| {
                match curr {
                    Some(x) => sum + x,
                    None => panic!("Cannot activate node.")
                }
            });
        self.activation.activate(total)
    }

    /// For backpropagation, return the derivative of neuron's activation
    /// function given the output of it's original calculation (activate)
    fn deactivate(&mut self, curr_value: f64) -> f64 {
        self.activation.deactivate(curr_value)
    }

}
