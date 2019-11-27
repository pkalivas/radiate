

use std::collections::HashMap;
use super::layer::Layer;
use super::neurons::{
    neuron::Neuron,
    dense::Dense
};
use super::activation::Activation;
use super::nodetype::NodeType;




#[derive(Debug)]
pub struct Vertex {
    pub innov: i32,
    pub outgoing: Vec<i32>,
    pub incoming: HashMap<i32, Option<f64>>,
    pub curr_value: Option<f64>,
    pub layer_type: Layer,
    pub neuron: Box<dyn Neuron>
}



impl Vertex {

    pub fn new(innov: i32, layer_type: Layer, node_type: NodeType, activation: Activation) -> Self {
        Vertex {
            innov,
            outgoing: Vec::new(),
            incoming: HashMap::new(),
            curr_value: None,
            layer_type,
            neuron: Vertex::neuron_factory(node_type, activation)
        }
    }

    pub fn as_mut_ptr(self) -> *mut Vertex {
        Box::into_raw(Box::new(self))
    }



    pub fn is_ready(&mut self) -> bool {
        self.incoming
            .values()
            .all(|x| x.is_some())
    }



    pub fn activate(&mut self) {
        self.curr_value = Some(self.neuron.activate(&self.incoming));
    }


    pub fn deactivate(&mut self) -> f64 {
        self.neuron.deactivate(self.curr_value.unwrap())
    }



    pub fn reset_neuron(&mut self) {
        self.curr_value = None;
        for (_, val) in self.incoming.iter_mut() {
            *val = None;
        }
        self.neuron.reset();
    }



    fn neuron_factory(node_type: NodeType, activation: Activation) -> Box<dyn Neuron> {
        match node_type {
            NodeType::Dense => Box::new(Dense { activation }),
            NodeType::LSTM => Box::new(Dense { activation }),
            NodeType::Recurrent => Box::new(Dense { activation })
        }
    }


}


impl Clone for Vertex {
    fn clone(&self) -> Self { 
        Vertex {
            innov: self.innov,
            outgoing: self.outgoing.iter().map(|x| *x).collect(),
            incoming: self.incoming.iter().map(|(key, val)| (*key, *val)).collect(),
            curr_value: self.curr_value.clone(),
            layer_type: self.layer_type.clone(),
            neuron: self.neuron.clone()
        }
    }
}


impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.innov == other.innov
    }
}