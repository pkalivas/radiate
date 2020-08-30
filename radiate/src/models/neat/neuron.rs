
extern crate rand;

use rand::Rng;

use super::id::*;
use super::edge::*;
use super::activation::Activation;
use super::neurontype::NeuronType;
use super::direction::NeuronDirection;


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NeuronLink {
    pub id: EdgeId,
    pub src: NeuronId,
    pub weight: f32,
}

impl NeuronLink {
    pub fn new(edge: &Edge) -> Self {
        Self {
            id: edge.id,
            src: edge.src,
            weight: edge.weight,
        }
    }
}

/// Neuron is a wrapper around a neuron providing only what is needed for a neuron to be added 
/// to the NEAT graph, while the neuron encapsulates the neural network logic for the specific node type,
/// Some neurons like an LSTM require more variables and different internal activation logic,
/// so encapsulating that within a normal node on the graph would be misplaced.
#[derive(Deserialize, Serialize, Debug)]
pub struct Neuron {
    pub id: NeuronId,
    outgoing: Vec<EdgeId>,
    incoming: Vec<NeuronLink>,
    activation: Activation,
    direction: NeuronDirection,
    pub neuron_type: NeuronType,
    pub activated_value: f32,
    pub deactivated_value: f32,
    pub current_state: f32,
    pub previous_state: f32,
    pub error: f32,
    pub bias: f32,
}


impl Neuron {
    pub fn new(id: NeuronId, neuron_type: NeuronType, activation: Activation, direction: NeuronDirection) -> Self {
        Neuron {
            id,
            outgoing: Vec::new(),
            incoming: Vec::new(),
            activation,
            neuron_type,
            direction,
            activated_value: 0.0,
            deactivated_value: 0.0,
            current_state: 0.0,
            previous_state: 0.0,
            error: 0.0,
            bias: rand::thread_rng().gen::<f32>(),
        }
    }

    /// Add incoming edge
    pub fn add_incoming(&mut self, edge: &Edge) {
        self.incoming.push(NeuronLink::new(edge));
    }

    /// Add outgoing edge
    pub fn add_outgoing(&mut self, edge: EdgeId) {
        self.outgoing.push(edge);
    }

    /// Update incoming edge
    pub fn update_incoming(&mut self, edge: &Edge, weight: f32) {
        if let Some(link) = self.incoming.iter_mut().find(|x| x.id == edge.id) {
            link.weight = weight;
        }
    }

    /// Remove incoming edge
    pub fn remove_incoming(&mut self, edge: &Edge) {
        self.incoming.retain(|x| x.id != edge.id);
    }

    /// Remove outgoing edge
    pub fn remove_outgoing(&mut self, edge: EdgeId) {
        self.outgoing.retain(|x| x != &edge);
    }

    /// Get incoming edge ids.
    pub fn incoming_edges(&self) -> &[NeuronLink] {
        &self.incoming
    }

    /// Get outgoing edge ids.
    pub fn outgoing_edges(&self) -> &[EdgeId] {
        &self.outgoing
    }

    /// 𝜎(Σ(w * i) + b)
    /// activate this node by calling the underlying neuron's logic for activation
    #[inline]
    pub fn activate(&mut self) {
        if self.activation != Activation::Softmax {
            match self.direction {
                NeuronDirection::Forward => {
                    self.activated_value = self.activation.activate(self.current_state);
                    self.deactivated_value = self.activation.deactivate(self.current_state);
                },
                NeuronDirection::Recurrent => {
                    self.activated_value = self.activation.activate(self.current_state + self.previous_state);
                    self.deactivated_value = self.activation.deactivate(self.current_state + self.previous_state);
                }
            }
            self.previous_state = self.current_state;
        }
    }


    /// each Neuron has a base layer of reset which needs to happen 
    /// but on top of that each neuron might need to do more internally
    #[inline]
    pub fn reset_neuron(&mut self) {
        self.error = 0.0;
        self.activated_value = 0.0;
        self.deactivated_value = 0.0;
        self.current_state = 0.0;
    }


    #[inline]
    pub fn clone_with_values(&self) -> Self {
        Neuron {
            id: self.id,
            outgoing: self.outgoing.clone(),
            incoming: self.incoming.clone(),
            current_state: self.current_state.clone(),
            previous_state: self.previous_state.clone(),
            activated_value: self.activated_value.clone(),
            deactivated_value: self.deactivated_value.clone(),
            error: self.error.clone(),
            bias: self.bias.clone(),
            activation: self.activation.clone(),
            neuron_type: self.neuron_type.clone(),
            direction: self.direction.clone()
        }
    }
}

impl Clone for Neuron {
    fn clone(&self) -> Self { 
        Neuron {
            id: self.id,
            outgoing: self.outgoing.clone(),
            incoming: self.incoming.clone(),
            current_state: 0.0,
            previous_state: 0.0,
            activated_value: 0.0,
            deactivated_value: 0.0,
            error: 0.0,
            bias: self.bias.clone(),
            activation: self.activation.clone(),
            neuron_type: self.neuron_type.clone(),
            direction: self.direction.clone()
        }
    }
}
