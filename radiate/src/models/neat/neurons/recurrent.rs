
extern crate rand;

use std::collections::HashMap;
use super::activation::Activation;
use super::neuron::Neuron;

#[derive(Debug)]
pub struct Recurrent {
    previous_incoming: HashMap<i32, f64>,
    activation: Activation

}


impl Recurrent {

    pub fn new(activation: Activation) -> Self {
        Recurrent {
            previous_incoming: HashMap::new(),
            activation
        }
    }

}


///
/// I'm not sure if this works?
/// https://peterroelants.github.io/posts/rnn-implementation-part01/
/// https://medium.com/@vsadhak/disassembling-recurrent-neural-networks-695ce75dddf6
///


impl Neuron for Recurrent {

    fn reset(&mut self) { }

    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64 {
        if self.previous_incoming.is_empty() {
            for innov in incoming.keys() {
                self.previous_incoming.insert(*innov, 0.0);
            }
        }
        let mut new_previous_inputs = HashMap::new();
        let total = incoming.iter()
            .fold(0.0, |sum, (innov, value)| {
                new_previous_inputs.insert(*innov, value.unwrap());
                sum + value.unwrap() + self.previous_incoming.get(innov).unwrap()
            });
        self.previous_incoming = new_previous_inputs;
        self.activation.activate(total)
    }

    fn deactivate(&mut self, curr_value: f64) -> f64 { 
        self.activation.deactivate(curr_value)
    }

}



impl Clone for Recurrent {
    fn clone(&self) -> Self {
        Recurrent {
            previous_incoming: HashMap::new(),
            activation: self.activation.clone()
        }
    }
}
