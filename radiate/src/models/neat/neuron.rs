

use std::collections::{HashMap};
use std::f64::consts::E as Eul;


/// In a neural network a node can be either an input, hidden
/// or output node. These mark the neruons as such
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeType {
    Input,
    Output,
    Hidden,
}


/// Varius activation functions for a neuron, must be specified at creation
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Activation {
    Sigmoid,
    Tahn,
    Relu,
    LeakyRelu(f64),
    ExpRelu(f64),
    Linear(f64)   
}


/// Neuron represents a node in a nerual network graph. It holds
/// an innovation number to help edges in the network identify which
/// node it's pointing to, a value which is its activated value 
/// a node type, being either input, hidden, or output, a vec of outgoing 
/// numbers. The output numbers are the innovation nmbers of the edges that
/// connect this node to another node (meaning this node is the egde's src node)
/// this lets us traverse the network quickly and simply while also keeping
/// track of the weights and active flags of the connections. Incoming keeps 
/// track of the nodes this node is expecting inputs from, the key is the innovation
/// number of the node it is expecting input from, and the value is that input
#[derive(Debug)]
pub struct Neuron {
    pub innov: i32,
    pub curr_value: Option<f64>,
    pub prev_value: Option<f64>,
    pub node_type: NodeType,
    pub activation: Activation,
    pub outgoing: Vec<i32>,
    pub incoming: HashMap<i32, Option<f64>>
}



/// implement the neuron
impl Neuron {

    /// return a blank neuron with only a innov and node type, everything
    /// else is completely empty
    pub fn new(innov: i32, node_type: NodeType, activation: Activation) -> Self {
        Neuron {
            innov,
            curr_value: None,
            prev_value: None,
            node_type,
            activation,
            outgoing: Vec::new(),
            incoming: HashMap::new()
        }
    }



    /// Turn the neuron into a raw mutable pointer - this
    /// makes the data structure inherintly unsafe 
    pub fn as_mut_ptr(self) -> *mut Neuron {
        Box::into_raw(Box::new(self))
    }


    /// Activate the neuron by testing to see if first it can be activated,
    /// meaning it has gotten all its expected inputs, and if it does 
    /// activate the sum of those inputs and assign it to the value of the neuron
    /// If the neuron was activated, return true, else false
    #[inline]
    pub fn is_ready(&mut self) -> bool {
        let can_activate = self.incoming.values().all(|x| x.is_some());
        if can_activate {
            let mut total = 0.0;
            for value in self.incoming.values() {
                match value {
                    Some(v) => total += v,
                    None => panic!("failed to activate.")
                }
            }
            self.curr_value = Some(Neuron::activate(self.activation, total));
            return true;
        }
        false
    }



    /// reset the values in the neurons incoming nodes and its value 
    #[inline]
    pub fn reset_node(&mut self) {
        self.prev_value = self.curr_value.clone();
        self.curr_value = None;
        for (_, val) in self.incoming.iter_mut() {
            *val = None;
        }
    }



    /// Generic activation functions for an neural network - note for a few
    /// of these an alpha parameter is needed when first assining the function
    #[inline]
    fn activate(activation_function: Activation, x: f64) -> f64 {
        // match the given activation function then calculate it and return the result 
        match activation_function {
            Activation::Sigmoid =>  1.0 / (1.0 + (-x * 4.9).exp()),
            Activation::Tahn => x.tanh(),
            Activation::Relu => if x > 0.0 { x } else { 0.0 },
            Activation::Linear(alpha) => alpha * x,
            Activation::LeakyRelu(alpha) => {
                let a = alpha * x;
                if a > x {
                    return a;
                } 
                x
            },
            Activation::ExpRelu(alpha) => {
                if x >= 0.0 {
                    return x;
                }
                alpha * (Eul.powf(x) - 1.0)
            }
        }
    }



    /// Deactivation functions for the activation neurons 
    #[inline]
    #[allow(dead_code)]
    pub fn deactivate(activation_function: Activation, x: f64) -> f64 {
        // match the activation function, compute the derivative and return it 
        match activation_function {
            Activation::Sigmoid => x * (1.0 - x),
            Activation::Tahn => 1.0 - (x).powf(2.0),
            Activation::Linear(alpha) => alpha,
                Activation::Relu => {
                    if x > 0.0 { 
                        return 1.0;
                    }
                    0.0
                },
            Activation::ExpRelu(alpha) => {
                if x > 0.0 {
                    return 1.0;
                }
                alpha * x.exp()
            },
            Activation::LeakyRelu(alpha) => {
                if x > 0.0 { 
                    return 1.0;
                } 
                alpha 
            },
        }
    }
    
    
}


impl Clone for Neuron {
    fn clone(&self) -> Self {
        Neuron {
            innov: self.innov,
            curr_value: None,
            prev_value: None,
            node_type: self.node_type,
            activation: self.activation,
            outgoing: self.outgoing
                .iter()
                .map(|x| *x)
                .collect::<Vec<_>>(),
            incoming: self.incoming
                .iter()
                .map(|(key, val)| (*key, *val))
                .collect::<HashMap<_, _>>()
        }
    }
}