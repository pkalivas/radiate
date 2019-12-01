

/// might be useful:
/// 
/// http://nn.cs.utexas.edu/downloads/papers/rawal.gecco2016.pdf
/// https://github.com/wagenaartje/neataptic
/// 



use std::collections::{HashMap, BTreeMap};
use super::activation::Activation;
use super::neuron::Neuron;

#[derive(Debug)]
pub struct LSTM {
    hidden_state: BTreeMap<i32, f64>,
    hidden_state_activated: BTreeMap<i32, f64>,
    forget_gate_weight: f64,
    update_gate_weight: f64,
    output_gate_weight: f64,
    activation: Activation
}


impl LSTM {

    pub fn new(activation: Activation) -> Self {
        LSTM {
            hidden_state: BTreeMap::new(),
            hidden_state_activated: BTreeMap::new(),
            forget_gate_weight: 0.0,
            update_gate_weight: 0.0,
            output_gate_weight: 0.0,
            activation
        }
    }

}


impl Neuron for LSTM {

    fn mutate(&mut self, should_edit: f32, size: f64) {

    }

    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64 {
        // calculate the forget gate of the neuron
        
        0.0
    }

    fn deactivate(&mut self, curr_value: f64) -> f64 {
        0.0
    }

}


impl Clone for LSTM {
    fn clone(&self) -> Self {
        LSTM {
            hidden_state: BTreeMap::new(),
            hidden_state_activated: BTreeMap::new(),
            forget_gate_weight: self.forget_gate_weight,
            update_gate_weight: self.update_gate_weight,
            output_gate_weight: self.output_gate_weight,
            activation: self.activation.clone()
        }
    }
}