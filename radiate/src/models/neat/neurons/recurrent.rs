
extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use super::activation::Activation;
use super::neuron::Neuron;

#[derive(Debug)]
pub struct Recurrent {
    hidden_inputs: HashMap<i32, f64>,
    hidden_weights: HashMap<i32, f64>,
    activation: Activation

}


impl Recurrent {

    pub fn new(activation: Activation) -> Self {
        Recurrent {
            hidden_inputs: HashMap::new(),
            hidden_weights: HashMap::new(),
            activation
        }
    }

}


///
/// I'm not sure if this works?
/// https://peterroelants.github.io/posts/rnn-implementation-part01/
/// https://medium.com/@vsadhak/disassembling-recurrent-neural-networks-695ce75dddf6
/// https://github.com/evolvingstuff/RecurrentJava
///


impl Neuron for Recurrent {

    fn mutate(&mut self, should_edit: f32, size: f64) {
        let mut r = rand::thread_rng();
        self.hidden_weights = self.hidden_weights.iter()
            .map(|(key, value)| {
                if r.gen::<f32>() < should_edit {
                    (*key, r.gen::<f64>())
                } else {
                    (*key, *value * r.gen_range(-size, size))
                }
            })
            .collect::<HashMap<_, _>>();
    }


    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64 {
        let mut new_hidden_inputs = HashMap::new();
        let total = incoming.iter()
            .fold(0.0, |sum, (innov, value)| {
                if !self.hidden_inputs.contains_key(innov) {
                    let mut r = rand::thread_rng();
                    self.hidden_inputs.insert(*innov, 0.0);
                    self.hidden_weights.insert(*innov, r.gen::<f64>());
                }
                let hidden_state = self.activation.activate(value.unwrap() + (self.hidden_inputs.get(innov).unwrap() * self.hidden_weights.get(innov).unwrap()));
                new_hidden_inputs.insert(*innov, hidden_state);
                sum + hidden_state
            });
        self.hidden_inputs = new_hidden_inputs;
        Activation::Sigmoid.activate(total)
    }


    fn deactivate(&mut self, curr_value: f64) -> f64 { 
        self.activation.deactivate(curr_value)
    }

}



impl Clone for Recurrent {
    fn clone(&self) -> Self {
        Recurrent {
            hidden_inputs: self.hidden_inputs.keys()
                .map(|x| (*x, 0.0))
                .collect::<HashMap<_, _>>(),
            hidden_weights: self.hidden_weights.iter()
                .map(|(key, value)| (*key, *value))
                .collect::<HashMap<_, _>>(),
            activation: self.activation.clone()
        }
    }
}
