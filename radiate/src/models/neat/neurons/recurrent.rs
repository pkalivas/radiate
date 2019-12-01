
extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use super::activation::Activation;
use super::neuron::Neuron;

#[derive(Debug)]
pub struct Recurrent {
    // hidden_inputs: HashMap<i32, f64>,
    // hidden_weights: HashMap<i32, f64>,
    hidden_input: f64,
    hidden_weight: f64,
    activation: Activation

}


impl Recurrent {

    pub fn new(activation: Activation) -> Self {
        Recurrent {
            // hidden_inputs: HashMap::new(),
            // hidden_weights: HashMap::new(),
            hidden_input: 0.0,
            hidden_weight: 0.0,
            activation
        }
    }

}


///
/// I'm not sure if this works?
/// https://peterroelants.github.io/posts/rnn-implementation-part01/
/// https://medium.com/@vsadhak/disassembling-recurrent-neural-networks-695ce75dddf6
/// https://github.com/JosephCatrambone/RRNN/blob/master/src/lib.rs
///


impl Neuron for Recurrent {

    fn mutate(&mut self, should_edit: f32, size: f64) {
        let mut r = rand::thread_rng();
        if r.gen::<f32>() < should_edit {
            self.hidden_weight = r.gen::<f64>()
        } else {
            self.hidden_weight *= r.gen_range(-size, size)
        };
    }


    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64 {
        let total = incoming.iter()
            .fold(0.0, |sum, (_, value)| {
                sum + value.unwrap()
            }) + (self.hidden_input * self.hidden_weight);
        self.hidden_input = self.activation.activate(total);
        self.hidden_input
    }


    fn deactivate(&mut self, curr_value: f64) -> f64 { 
        self.activation.deactivate(curr_value)
    }

}



impl Clone for Recurrent {
    fn clone(&self) -> Self {
        Recurrent {
            hidden_input: 0.0,
            hidden_weight: self.hidden_weight,
            activation: self.activation.clone()
        }
    }
}
