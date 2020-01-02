extern crate rand;

use std::error::Error;
use std::sync::{Arc, RwLock};
use super::{
    neatenv::NeatEnvironment,
    activation::Activation,
    layers::{
        layer::Layer,
        dense::Dense,
        lstm::LSTM,
        gru::GRU,
        layertype::LayerType,
    }
};

use crate::engine::genome::Genome;



#[derive(Debug)]
pub struct LayerWrap {
    pub layer_type: LayerType,
    pub layer: Box<dyn Layer>
}


impl LayerWrap {
    pub fn as_ref<L: Layer>(&self) -> &L {
        self.layer.as_ref_any().downcast_ref::<L>().unwrap()
    }

    pub fn as_mut<L: Layer>(&mut self) -> &mut L {
        self.layer.as_mut_any().downcast_mut::<L>().unwrap()
    }
}



/// Neat is a neural network consisting of layers
/// the layers can be stacked together then the feed forward
/// and backprop functions will take care of 'connecting them'
#[derive(Debug)]
pub struct Neat {
    pub layers: Vec<LayerWrap>,
    pub input_size: u32
}



impl Neat {

    
    pub fn new() -> Self {
        Neat { 
            layers: Vec::new(),
            input_size: 0
        }
    }



    /// set the input size for the network 
    pub fn input_size(mut self, input_size: u32) -> Self {
        self.input_size = input_size;
        self
    }



    /// train the network
    #[inline]
    pub fn train(&mut self, inputs: &Vec<Vec<f32>>, targets: &Vec<Vec<f32>>, iters: usize, rate: f32, update_window: usize) -> Result<(), Box<dyn Error>> {
        // make sure the data actually can be fed through
        assert!(inputs.len() == targets.len(), "Input and target data are different sizes");
        assert!(inputs[0].len() as u32 == self.input_size, "Input size is different than network input size");
        
        // feed the input data through the network then back prop it back through to edit the weights of the layers
        for _ in 0..iters {
            for (index, (input, target)) in inputs.iter().zip(targets.iter()).enumerate() {
                let network_output = self.forward(input).ok_or("Error in network feed forward")?;
                if index + 1 % update_window == 0 {
                    self.backward(&network_output, &target, rate, true);
                } else {
                    self.backward(&network_output, &target, rate, false);
                }
            }
        }
        Ok(())
    }



    /// backprop the the error of the network through each layer adjusting the weights
    #[inline]
    pub fn backward(&mut self, network_output: &Vec<f32>, target: &Vec<f32>, learning_rate: f32, update: bool) {
        // pass back the errors from this output layer through the network to update either the optimizer of the weights of the network
        self.layers
            .iter_mut()
            .rev()
            .fold(target
                    .iter()
                    .zip(network_output.iter())
                    .map(|(tar, pre)| tar - pre)
                    .collect(), |res, curr| {
                curr.layer.backward(&res, learning_rate, update).unwrap()
            });
    }
    


    /// feed forward a vec of data through the neat network 
    #[inline]
    pub fn forward(&mut self, data: &Vec<f32>) -> Option<Vec<f32>> {
        // keep two vec in order to transfer the data from one layer to another layer in the network
        let mut temp;
        let mut data_transfer = data;
        for wrapper in self.layers.iter_mut() {
            temp = wrapper.layer.forward(data_transfer)?;
            data_transfer = &temp;
        }
        // gather the output and return it as an option
        let output = data_transfer
            .iter()
            .map(|x| *x)
            .collect();
        Some(output)
    }    



    /// create and append a new dense pool layer onto the neat network
    #[inline]
    pub fn dense_pool(mut self, size: u32, activation: Activation) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(size).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::DensePool,
            layer: Box::new(Dense::new(input_size, output_size, LayerType::DensePool, activation))
        };
        self.layers.push(wrapper);
        self
    }



    /// create an append a simple dense layer onto the network
    #[inline]
    pub fn dense(mut self, size: u32, activation: Activation) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(size).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::Dense,
            layer: Box::new(Dense::new(input_size, output_size, LayerType::Dense, activation))
        };
        self.layers.push(wrapper);
        self
    }


    
    /// create a new lstm layer and add it to the network
    #[inline]
    pub fn lstm(mut self, size: u32) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(size).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::LSTM,
            layer: Box::new(LSTM::new(input_size, output_size))
        };
        self.layers.push(wrapper);
        self
    }



    /// create a new lstm layer and add it to the network
    #[inline]
    pub fn gru(mut self, size: u32, output_size: u32) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(output_size).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::GRU,
            layer: Box::new(GRU::new(input_size, size, output_size))
        };
        self.layers.push(wrapper);
        self
    }

    

    /// in order to more efficently give inputs to the network, this function simple 
    /// finds the shape of the layer that should be created based on the desired size
    #[inline]
    fn get_layer_sizes(&self, size: u32) -> Option<(u32, u32)> {
        if self.layers.len() == 0 {
            return Some((self.input_size, size))
        } 
        Some((self.layers.last()?.layer.shape().1 as u32, size))
    }

}



/// Implement clone for the neat neural network in order to facilitate 
/// proper crossover and mutation for the network
impl Clone for Neat {
    fn clone(&self) -> Self {
        Neat {
            layers: self.layers
                .iter()
                .map(|x| {
                    LayerWrap { 
                        layer_type: x.layer_type.clone(), 
                        layer: x.layer.clone() 
                    }
                })
                .collect(),
            input_size: self.input_size
        }
    }
}
/// These must be implemneted for the network or any type to be 
/// used within seperate threads. Because implementing the functions 
/// themselves is dangerious and unsafe and i'm not smart enough 
/// to do that from scratch, these "implmenetaions" will get rid 
/// of the error and realistically they don't need to be implemneted for the
/// program to work
unsafe impl Send for Neat {}
unsafe impl Sync for Neat {}
/// Implement partialeq for neat because if neat itself is to be used as a problem,
/// it must be able to compare one to another
impl PartialEq for Neat {
    fn eq(&self, other: &Self) -> bool {
        for (one, two) in self.layers.iter().zip(other.layers.iter()) {
            if &one.layer != &two.layer {
                return false;
            }
        }
        true
    }
}



/// iplement genome for a neat network
impl Genome<Neat, NeatEnvironment> for Neat {

    #[inline]
    fn crossover(one: &Neat, two: &Neat, env: &Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<Neat> {
        let mut result_layers = Vec::with_capacity(one.layers.len());
        // iterate through the layers of the network and cross them over with each other
        for (one_layer, two_layer) in one.layers.iter().zip(two.layers.iter()) {
            let new_layer: Box<dyn Layer> = match one_layer.layer_type {
                LayerType::Dense | LayerType::DensePool => {
                    Box::new(Dense::crossover(one_layer.as_ref(), two_layer.as_ref(), env, crossover_rate)?)
                },
                LayerType::LSTM => {
                    Box::new(LSTM::crossover(one_layer.as_ref(), two_layer.as_ref(), env, crossover_rate)?)
                },
                LayerType::GRU => {
                    Box::new(GRU::crossover(one_layer.as_ref(), two_layer.as_ref(), env, crossover_rate)?)
                }
            };

            result_layers.push(LayerWrap {
                layer_type: one_layer.layer_type,
                layer: new_layer
            });
        }
        Some(Neat { layers: result_layers, input_size: one.input_size })
    }



    fn base(env: &mut NeatEnvironment) -> Neat {
        Neat::new().input_size(env.input_size.unwrap()).dense_pool(env.output_size.unwrap(), Activation::Sigmoid)
    }



    fn distance(one: &Neat, two: &Neat, env: &Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut total_distance = 0.0;
        for (layer_one, layer_two) in one.layers.iter().zip(two.layers.iter()) {
            total_distance += match layer_one.layer_type {
                LayerType::Dense | LayerType::DensePool => {
                    Dense::distance(layer_one.as_ref(), layer_two.as_ref(), env)
                },
                LayerType::LSTM => {
                    LSTM::distance(layer_one.as_ref(), layer_two.as_ref(), env)
                },
                LayerType::GRU => {
                    GRU::distance(layer_one.as_ref(), layer_two.as_ref(), env)
                }
            };
        }
        total_distance
    }

}
