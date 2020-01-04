
extern crate uuid;

use std::collections::HashMap;
use uuid::Uuid;


#[derive(Debug)]
pub struct Tracer {
    pub neuron_states: HashMap<Uuid, Vec<f32>>,
    pub edge_states: HashMap<Uuid, Vec<f32>>,
    pub max_neuron_index: usize,
    pub max_edge_index: usize,
    pub index: usize,
}



impl Tracer {

    pub fn new() -> Self {
        Tracer {
            neuron_states: HashMap::new(),
            edge_states: HashMap::new(),
            max_neuron_index: 0,
            max_edge_index: 0,
            index: 0,
        }
    }


    pub fn set_index(&mut self, new_index: usize) {
        assert!(new_index <= self.index, "New index is larger than self index");
        self.index = new_index;
    }


    pub fn reset(&mut self) {
        self.neuron_states = HashMap::new();
        self.edge_states = HashMap::new();
        self.index = 0;        
    }


    pub fn update_neuron(&mut self, neuron_id: Uuid, neuron_state: f32) {
        if self.neuron_states.contains_key(&neuron_id) {
            let states = self.neuron_states.get_mut(&neuron_id).unwrap();
            states.push(neuron_state);
            if states.len() > self.max_neuron_index {
                self.max_neuron_index += 1;
            }
        } else {
            let mut temp = Vec::with_capacity(self.max_neuron_index);
            temp.push(neuron_state);
            self.neuron_states.insert(neuron_id, temp);
        }
    }


    pub fn update_edge(&mut self, edge_id: Uuid, edge_state: f32) {
        if self.edge_states.contains_key(&edge_id) {
            let states = self.edge_states.get_mut(&edge_id).unwrap();
            states.push(edge_state);
            if states.len() > self.max_edge_index {
                self.max_edge_index += 1;
            }
        } else {
            let mut temp = Vec::with_capacity(self.max_edge_index);
            temp.push(edge_state);
            self.edge_states.insert(edge_id, temp);
        }
    }


    pub fn neuron_state(&self, neuron_id: Uuid) -> f32 {
        if !self.neuron_states.contains_key(&neuron_id) {
            panic!("Tracer neuron state doesn't contain uuid: {:?}", neuron_id);
        }
        self.neuron_states.get(&neuron_id).unwrap()[self.index]
    }


    pub fn edge_state(&self, edge_id: Uuid) -> f32 {
        if !self.edge_states.contains_key(&edge_id) {
            panic!("Tracer edge state doesn't contain uuid: {:?}", edge_id);
        }
        self.edge_states.get(&edge_id).unwrap()[self.index]
    }

}