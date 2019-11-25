

use std::collections::HashMap;
use super::layer::Layer;
use super::activation::Activation;
use super::neur::Neue;


#[derive(Debug)]
pub struct Dense {
    pub innov: i32,
    pub layer_type: Layer,
    pub activation: Activation,
    pub curr_value: Option<f64>,
    pub outgoing: Vec<i32>,
    pub incoming: HashMap<i32, Option<f64>>
}


impl Dense {

    pub fn new(innov: i32, layer_type: Layer, activation: Activation) -> Self {
        Dense {
            innov,
            layer_type,
            activation,
            curr_value: None,
            outgoing: Vec::new(),
            incoming: HashMap::new()
        }
    }

}


impl Neue for Dense {

    fn is_ready(&mut self) -> bool {
        let can_activate = self.incoming.values().all(|x| x.is_some());
        if can_activate {
            let mut total = 0.0;
            for value in self.incoming.values() {
                match value {
                    Some(v) => total += v,
                    None => panic!("failed to activate.")
                }
            }
            self.curr_value = Some(self.activation.activate(total));
            return true;
        }
        false
    }


    /// reset the values in the neurons incoming nodes and its value 
    #[inline]
    fn reset(&mut self) {
        self.curr_value = None;
        for (_, val) in self.incoming.iter_mut() {
            *val = None;
        }
    }

}