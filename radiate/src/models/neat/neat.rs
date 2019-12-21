extern crate rand;

use std::sync::{Arc, RwLock};
use super::{
    neatenv::NeatEnvironment,
    activation::Activation,
    layers::{
        layer::Layer,
        dense::Dense,
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
    pub layers: Vec<LayerWrap>
}



impl Neat {

    
    pub fn new() -> Self {
        Neat { layers: Vec::new() }
    }

    

    /// feed forward a vec of data through the neat network 
    #[inline]
    pub fn feed_forward(&mut self, data: &Vec<f64>) -> Option<Vec<f64>> {
        // keep two vec in order to transfer the data from one layer to another layer in the network
        let mut temp;
        let mut data_transfer = data;
        for wrapper in self.layers.iter_mut() {
            temp = wrapper.layer.propagate(data_transfer)?;
            data_transfer = &temp;
        }
        // gather the output and return it as an option
        let output = data_transfer
            .iter()
            .map(|x| *x)
            .collect();
        Some(output)
    }

    

    /// backprop the the error of the network through each layer adjusting the weights
    #[inline]
    pub fn backprop(&mut self, data: &Vec<f64>, output: &Vec<f64>, learning_rate: f64) {
        // feed forward the input data to set the outputs of each neuron in the network
        // compute the original errors
        let feed_out = self.feed_forward(data).unwrap();
        let errors = output
            .iter()
            .zip(feed_out.iter())
            .map(|(target, prediction)| {
                target - prediction
            })
            .collect::<Vec<_>>();

        // similar to feed_forward, keep mutable vecs in order to transfer the error
        // from the one layer back to the layer preceding it
        self.layers
            .iter_mut()
            .rev()
            .fold(errors, |res, curr| {
                curr.layer.backprop(&res, learning_rate).unwrap()
            });
    }



    /// create and append a new dense pool layer onto the neat network
    #[inline]
    pub fn dense_pool(mut self, size: i32, env: &mut NeatEnvironment, activation: Activation) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(size, env).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::DensePool,
            layer: Box::new(Dense::new(input_size, output_size, LayerType::DensePool, activation))
        };
        self.layers.push(wrapper);
        self
    }



    /// create an append a simple dense layer onto the network
    #[inline]
    pub fn dense(mut self, size: i32, env: &mut NeatEnvironment, activation: Activation) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(size, env).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::Dense,
            layer: Box::new(Dense::new(input_size, output_size, LayerType::Dense, activation))
        };
        self.layers.push(wrapper);
        self
    }

    

    /// in order to more efficently give inputs to the network, this function simple 
    /// finds the shape of the layer that should be created based on the desired size
    #[inline]
    fn get_layer_sizes(&self, size: i32, env: &mut NeatEnvironment) -> Option<(i32, i32)> {
        if self.layers.len() == 0 {
            return Some((env.input_size?, size))
        } 
        Some((self.layers.last()?.layer.shape().1 as i32, size))
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
                .collect()
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
            let new_layer = match one_layer.layer_type {
                LayerType::Dense | LayerType::DensePool => {
                    Dense::crossover(one_layer.as_ref(), two_layer.as_ref(), env, crossover_rate)?
                },
                _ => panic!("Layer Type not implemented")
            };

            result_layers.push(LayerWrap {
                layer_type: one_layer.layer_type,
                layer: Box::new(new_layer)
            });
        }
        Some(Neat { layers: result_layers })
    }



    fn base(env: &mut NeatEnvironment) -> Neat {
        Neat::new()
            .dense_pool(env.output_size.unwrap(), env, Activation::Sigmoid)
    }



    fn distance(one: &Neat, two: &Neat, env: &Arc<RwLock<NeatEnvironment>>) -> f64 {
        let mut total_distance = 0.0;
        for (layer_one, layer_two) in one.layers.iter().zip(two.layers.iter()) {
            total_distance += match layer_one.layer_type {
                LayerType::Dense | LayerType::DensePool => {
                    Dense::distance(layer_one.as_ref(), layer_two.as_ref(), env)
                },
                _ => panic!("Layer Type not implemented")
            };
        }
        total_distance
    }

}
