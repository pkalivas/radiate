
extern crate rand;
extern crate uuid;

use std::collections::HashMap;
use rand::Rng;
use uuid::Uuid;

use super::activation::Activation;
use super::neurontype::NeuronType;
use super::direction::NeuronDirection;


#[derive(Deserialize, Serialize, Debug)]
pub struct Tracker {
    pub current_value: f32,
    pub current_dvalue: f32,
    pub current_state: f32,
    pub previous_activations: Vec<f32>,
    pub previous_derivatives: Vec<f32>,
    pub previous_states: Vec<f32>,
    pub index: usize
}


impl Tracker {

    pub fn new() -> Self {
        Tracker {
            current_value: 0.0,
            current_dvalue: 0.0,
            current_state: 0.0,
            previous_activations: Vec::new(),
            previous_derivatives: Vec::new(),
            previous_states: Vec::new(),
            index: 0
        }
    }


    pub fn get_previous_state(&self) -> f32 {
        if self.previous_states.is_empty() {
            return 0.0
        }
        *self.previous_states.last().unwrap()
    }

}



/// Neuron is a wrapper around a neuron providing only what is needed for a neuron to be added 
/// to the NEAT graph, while the neuron encapsulates the neural network logic for the specific nodetype,
/// Some neurons like an LSTM require more variables and different interal activation logic, 
/// so encapsulating that within a normal node on the graph would be misplaced.
#[derive(Deserialize, Serialize, Debug)]
pub struct Neuron {
    pub innov: Uuid,
    pub outgoing: Vec<Uuid>,
    pub incoming: HashMap<Uuid, Option<f32>>,
    pub tracker: Tracker,
    pub direction: NeuronDirection,
    pub activation: Activation,
    pub neuron_type: NeuronType,
    pub bias: f32,
    pub error: f32,
}



impl Neuron {


    pub fn new(innov: Uuid, neuron_type: NeuronType, activation: Activation, direction: NeuronDirection) -> Self {
        Neuron {
            innov,
            outgoing: Vec::new(),
            incoming: HashMap::new(),
            tracker: Tracker::new(),
            direction,
            activation,
            neuron_type,
            bias: rand::thread_rng().gen::<f32>(),
            error: 0.0,
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



    pub fn get_step(&self) -> f32 {
        self.error * self.tracker.previous_derivatives[self.tracker.index - 1]
    }


    pub fn get_activation(&self) -> f32 {
        self.tracker.previous_activations[self.tracker.index - 1]
    }


    
    /// 𝜎(Σ(w * i) + b)
    /// activate this node by calling the underlying neuron's logic for activation
    /// given the hashmap of <incoming edge innov, Option<incoming Neuron output value>>
    #[inline]
    pub fn activate(&mut self) {
        let state = self.incoming
            .values()
            .fold(self.bias, |sum, curr| {
                match curr {
                    Some(x) => sum + x,
                    None => panic!("Cannot activate node.")
                }
            });

        if self.activation != Activation::Softmax {
            match self.direction {
                NeuronDirection::Forward => {
                    self.tracker.current_value = self.activation.activate(state);
                    self.tracker.current_dvalue = self.activation.deactivate(state);
                },
                NeuronDirection::Recurrent => {
                    self.tracker.current_value = self.activation.activate(state + self.tracker.get_previous_state());
                    self.tracker.current_dvalue = self.activation.deactivate(state + self.tracker.get_previous_state());
                }
            }
        }

        self.tracker.previous_activations.push(self.tracker.current_value);
        self.tracker.previous_derivatives.push(self.tracker.current_dvalue);
        self.tracker.previous_states.push(self.tracker.current_state);
        self.tracker.current_state = state;
        self.tracker.index += 1;
    }



    /// each Neuron has a base layer of reset which needs to happen 
    /// but on top of that each neuron might need to do more interanally
    #[inline]
    pub fn reset_neuron(&mut self) {
        self.error = 0.0;
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
                .map(|(key, _)| (*key, None))
                .collect(),
            tracker: Tracker::new(),
            direction: self.direction,
            bias: self.bias.clone(),
            error: 0.0,
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
