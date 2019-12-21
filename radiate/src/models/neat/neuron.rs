
extern crate rand;
extern crate uuid;

use std::fmt;
use std::collections::HashMap;
use rand::Rng;
use uuid::Uuid;
use super::activation::Activation;
use super::neurontype::NeuronType;




/// Neuron is a wrapper around a neuron providing only what is needed for a neuron to be added 
/// to the NEAT graph, while the neuron encapsulates the neural network logic for the specific nodetype,
/// Some neurons like an LSTM require more variables and different interal activation logic, 
/// so encapsulating that within a normal node on the graph would be misplaced.
pub struct Neuron {
    pub innov: Uuid,
    pub outgoing: Vec<Uuid>,
    pub incoming: HashMap<Uuid, Option<f64>>,
    pub value: Option<f64>,
    pub error: Option<f64>,
    pub bias: f64,
    pub activation: Activation,
    pub neuron_type: NeuronType
}



impl Neuron {


    pub fn new(innov: Uuid, neuron_type: NeuronType, activation: Activation) -> Self {
        Neuron {
            innov,
            outgoing: Vec::new(),
            incoming: HashMap::new(),
            value: None,
            error: None,
            bias: rand::thread_rng().gen::<f64>(),
            activation,
            neuron_type,
        }
    }



    /// Return this struct as a raw mutable pointer - consumes the struct
    pub fn as_mut_ptr(self) -> *mut Neuron {
        Box::into_raw(Box::new(self))
    }



    /// figure out if this node can be calculated, meaning all of the 
    /// nodes pointing to it have given this node their output values.
    /// If they have, this node is ready to be activated
    #[inline]
    pub fn is_ready(&mut self) -> bool {
        self.incoming
            .values()
            .all(|x| x.is_some())
    }


    
    /// activate this node by calling the underlying neuron's logic for activation
    /// given the hashmap of <incoming edge innov, Option<incoming Neuron output value>>
    #[inline]
    pub fn activate(&mut self) {
        let total = self.incoming
            .values()
            .fold(self.bias, |sum, curr| {
                match curr {
                    Some(x) => sum + x,
                    None => panic!("Cannot activate node.")
                }
            });
        self.value = Some(self.activation.activate(total));
    }



    /// deactivate this node by calling the underlying neuron's logic to compute
    /// the gradient of the original output value 
    #[inline]
    pub fn deactivate(&mut self) -> f64 {
        match self.value {
            Some(val) => self.activation.deactivate(val),
            None => panic!("Failed to deactivate neuron.")
        }
    }



    /// each Neuron has a base layer of reset which needs to happen 
    /// but on top of that each neuron might need to do more interanally
    #[inline]
    pub fn reset_neuron(&mut self) {
        self.value = None;
        self.error = None;
        for (_, val) in self.incoming.iter_mut() {
            *val = None;
        }
    }


}



impl Clone for Neuron {
    fn clone(&self) -> Self { 
        Neuron {
            innov: self.innov,
            outgoing: self.outgoing
                .iter()
                .map(|x| *x)
                .collect(),
            incoming: self.incoming
                .iter()
                .map(|(key, val)| (*key, *val))
                .collect(),
            value: self.value.clone(),
            error: self.error.clone(),
            bias: self.bias.clone(),
            activation: self.activation.clone(),
            neuron_type: self.neuron_type.clone(),
        }
    }
}


impl PartialEq for Neuron {
    fn eq(&self, other: &Self) -> bool {
        self.innov == other.innov
    }
}


impl fmt::Debug for Neuron {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let mut income = self.incoming
            .iter()
            .map(|x| format!("\n\t\t{} val -> {:?},", x.0, x.1))
            .collect::<Vec<_>>()
            .join("");
        let mut outcome = self.outgoing
            .iter()
            .map(|x| {
                format!("\n\t\t{},", x)
            })
            .collect::<Vec<_>>()
            .join("");   

        if self.incoming.len() > 0 {
            income.push_str("\n\t");
        }
        if self.outgoing.len() > 0 {
            outcome.push_str("\n\t");
        }

        write!(f, "{}\n\tinnov: {}\n\tvalue: {:?}\n\terror: {:?}\n\toutgoing: [{}]\n\tincoming: [{}]\n\ttype: {:?}\n\tactivation: {:?}\n{}",
            "{", self.innov, self.value, self.error, outcome, income, self.neuron_type, self.activation, "}") 
    }
}
