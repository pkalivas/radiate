
extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use super::activation::Activation;
use super::neuron::Neuron;


#[derive(Debug, Clone)]
pub struct Recurrent {
    previous_values: HashMap<i32, f64>,
    previous_weights: HashMap<i32, f64>,
    activation: Activation

}


impl Recurrent {

    pub fn new(activation: Activation) -> Self {
        Recurrent {
            previous_values: HashMap::new(),
            previous_weights: HashMap::new(),
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
        if self.previous_values.is_empty() {
            let mut r = rand::thread_rng();
            for innov in incoming.keys() {
                self.previous_values.insert(*innov, 0.0);
                self.previous_weights.insert(*innov, r.gen::<f64>());
            }
        }

        let mut hidden_total = 0.0;
        let new_hidden_state = self.previous_values.iter()
            .map(|(key, val)| {
                let x = val * self.previous_weights.get(key).unwrap();
                hidden_total += x;
                (*key, x)
            })
            .collect::<HashMap<_, _>>();

        let total = incoming.values()
            .fold(0.0, |sum, curr| sum + curr.unwrap());

        self.previous_values = new_hidden_state;
        self.activation.activate(total + hidden_total)
    }

    fn deactivate(&mut self, curr_value: f64) -> f64 { curr_value }

}
