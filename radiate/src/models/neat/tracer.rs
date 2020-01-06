
extern crate uuid;

use std::collections::HashMap;
use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct Tracer {
    pub neuron_activation: HashMap<Uuid, Vec<f32>>,
    pub neuron_derivative: HashMap<Uuid, Vec<f32>>,
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


    pub fn set_index(&mut self, new_index: usize) {
        assert!(new_index <= self.index, "New index is larger than self index");
        self.index = new_index;
    }


    pub fn incremnt_tracer(&mut self) {
        self.index += 1;
    }


    pub fn reset(&mut self) {
        self.neuron_activation = HashMap::new();
        self.neuron_derivative = HashMap::new();
        self.index = 0;        
    }



    pub fn update_neuron_activation(&mut self, neuron_id: Uuid, neuron_value: f32) {
        if self.neuron_activation.contains_key(&neuron_id) {
            let states = self.neuron_activation.get_mut(&neuron_id).unwrap();
            states.push(neuron_value);
            if states.len() > self.max_neuron_index {
                self.max_neuron_index += 1;
            }
        } else {
            let mut temp = Vec::with_capacity(self.max_neuron_index);
            temp.push(neuron_value);
            self.neuron_activation.insert(neuron_id, temp);
        }
    }



    pub fn update_neuron_derivative(&mut self, neuron_id: Uuid, neuron_d: f32) {
        if self.neuron_derivative.contains_key(&neuron_id) {
            let states = self.neuron_derivative.get_mut(&neuron_id).unwrap();
            states.push(neuron_d);
        } else {
            let mut temp = Vec::with_capacity(self.max_neuron_index);
            temp.push(neuron_d);
            self.neuron_derivative.insert(neuron_id, temp);
        }
    }



    pub fn neuron_activation(&self, neuron_id: Uuid) -> f32 {
        if !self.neuron_activation.contains_key(&neuron_id) {
            panic!("Tracer neuron state doesn't contain uuid: {:?}", neuron_id);
        }
        self.neuron_activation.get(&neuron_id).unwrap()[self.index]
    }


    pub fn neuron_derivative(&self, neuron_id: Uuid) -> f32 {
        if !self.neuron_derivative.contains_key(&neuron_id) {
            panic!("Tracer neuron state doesn't contain uuid: {:?}", neuron_id);
        }
        self.neuron_derivative.get(&neuron_id).unwrap()[self.index]
    }



}