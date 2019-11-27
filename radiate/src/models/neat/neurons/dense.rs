

use std::collections::HashMap;
use super::Activation;
use super::neuron::Neuron;



#[derive(Debug, Clone)]
pub struct Dense {
    pub activation: Activation,
}


impl Dense {

    pub fn new(activation: Activation) -> Self {
        Dense { activation }
    }

}


impl Neuron for Dense {

    fn reset(&mut self) { }


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


    fn deactivate(&mut self, curr_value: f64) -> f64 {
        self.activation.deactivate(curr_value)
    }

}
