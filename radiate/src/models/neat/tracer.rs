
use std::collections::HashMap;

use super::id::*;


/// Tracer keeps track of historical metadata for neurons to keep track
/// of their activated values and derivatives so backpropagation (through time)
/// is available for batch processing and weight updates
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tracer {
    pub neuron_activation: HashMap<NeuronId, Vec<f32>>,
    pub neuron_derivative: HashMap<NeuronId, Vec<f32>>,
    pub max_neuron_index: usize,
    pub index: usize,
}



impl Tracer {

    pub fn new() -> Self {
        Tracer {
            neuron_activation: HashMap::new(),
            neuron_derivative: HashMap::new(),        
            max_neuron_index: 0,
            index: 0,
        }
    }



    /// reset the tracer. The backprop works off of indexed values so when the
    /// layer is reset, the tracer must be reset as well
    pub fn reset(&mut self) {
        self.neuron_activation = HashMap::new();
        self.neuron_derivative = HashMap::new();
        self.index = 0;        
    }



    /// update a neuron and add it's activated value 𝜎(Σ(w * i) + b)
    pub fn update_neuron_activation(&mut self, neuron_id: &NeuronId, neuron_value: f32) {
        if self.neuron_activation.contains_key(&neuron_id) {
            let states = self.neuron_activation.get_mut(&neuron_id).unwrap();
            states.push(neuron_value);

            // keep track of how many values are being kept track of so the list's don't 
            // have to resize after one iteration, speeds things up as time goes on 
            if states.len() > self.max_neuron_index {
                self.max_neuron_index += 1;
            }
        } else {
            let mut temp = Vec::with_capacity(self.max_neuron_index);
            temp.push(neuron_value);
            self.neuron_activation.insert(*neuron_id, temp);
        }
    }



    /// update a neuron and add it's derivative of it's activated value to the tracer
    pub fn update_neuron_derivative(&mut self, neuron_id: &NeuronId, neuron_d: f32) {
        if self.neuron_derivative.contains_key(&neuron_id) {
            let states = self.neuron_derivative.get_mut(&neuron_id).unwrap();
            states.push(neuron_d);
        } else {
            let mut temp = Vec::with_capacity(self.max_neuron_index);
            temp.push(neuron_d);
            self.neuron_derivative.insert(*neuron_id, temp);
        }
    }



    /// return the activated value of a neuron at the current index 
    pub fn neuron_activation(&self, neuron_id: NeuronId) -> f32 {
        if !self.neuron_activation.contains_key(&neuron_id) {
            panic!("Tracer neuron state doesn't contain uuid: {:?}", neuron_id);
        }
        self.neuron_activation.get(&neuron_id).unwrap()[self.index - 1]
    }



    /// return the derivative of a neuron at the current index 
    pub fn neuron_derivative(&self, neuron_id: NeuronId) -> f32 {
        if !self.neuron_derivative.contains_key(&neuron_id) {
            panic!("Tracer neuron state doesn't contain uuid: {:?}", neuron_id);
        }
        self.neuron_derivative.get(&neuron_id).unwrap()[self.index - 1]
    }



}
