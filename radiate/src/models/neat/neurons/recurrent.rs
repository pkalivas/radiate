
use std::collections::HashMap;
use super::activation::Activation;
use super::neuron::Neuron;

#[derive(Debug, Clone)]
pub struct Recurrent {
    previous_value: f64,
    activation: Activation
}


impl Recurrent {

    pub fn new(activation: Activation) -> Self {
        Recurrent {
            previous_value: 0.0,
            activation
        }
    }

}


///
/// I'm not sure if this works?
///


impl Neuron for Recurrent {

    fn reset(&mut self) { }

    fn activate(&mut self, incoming: &HashMap<i32, Option<f64>>) -> f64 {
        let total = incoming.values()
            .fold(0.0, |sum, curr| {
                match curr {
                    Some(x) => sum + x,
                    None => panic!("Failed to activate Recurrent neuron")
                }
            }) + self.previous_value;
        let result = self.activation.activate(total);
        self.previous_value = result;
        result
    }

    fn deactivate(&mut self, curr_value: f64) -> f64 { 0.0 }

}


