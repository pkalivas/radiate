extern crate rand;
extern crate serde_json;

use std::fs::File;
use std::error::Error;
use std::sync::{Arc, RwLock};

use super::{
    neatenv::NeatEnvironment,
    activation::Activation,
    loss::Loss,
    layers::{
        layer::Layer,
        dense::Dense,
        lstm::LSTM,
        gru::GRU,
        layertype::LayerType,
        vectorops
    }
};

use crate::engine::genome::Genome;




#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Neat {
    pub layers: Vec<LayerWrap>,
    pub input_size: u32,
    pub batch_size: usize
}



impl Neat {

    
    pub fn new() -> Self {
        Neat { 
            layers: Vec::new(),
            input_size: 0,
            batch_size: 1
        }
    }



    /// set the input size for the network 
    pub fn input_size(mut self, input_size: u32) -> Self {
        self.input_size = input_size;
        self
    }



    /// set the batch size for the network
    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }



    /// reset the layers on the network
    pub fn reset(&mut self) {
        for l in self.layers.iter_mut() {
            l.layer.reset();
        }
    }



    /// train the network
    #[inline]
    pub fn train<F>(&mut self, inputs: &[Vec<f32>], targets: &[Vec<f32>], rate: f32, loss_fn: Loss, run: F) -> Result<(), Box<dyn Error>>
        where F: Fn(usize, f32) -> bool 
    {
        // make sure the data actually can be fed through
        assert!(inputs.len() == targets.len(), "Input and target data are different sizes");
        assert!(inputs[0].len() as u32 == self.input_size, "Input size is different than network input size");

        // feed the input data through the network then back prop it back through to edit the weights of the layers
        let mut pass_out = Vec::with_capacity(self.batch_size);
        let mut pass_tar = Vec::with_capacity(self.batch_size);
        let (mut epoch, mut count, mut loss) = (0, 0, 0.0);
        
        // add tracers to the layers during training to keep track of meta data for backprop
        if self.batch_size > 1 {
            self.layers
                .iter_mut()
                .for_each(|x| x.layer.add_tracer());
        }
        
        // iterate through the number of iterations and train the network
        loop {
            for j in 0..inputs.len() {
                count += 1;
                pass_out.push(self.forward(&inputs[j]).ok_or("Error in network feed forward")?);
                pass_tar.push(targets[j].clone());
                if count == self.batch_size || j == inputs.len() - 1 {
                    count = 0;
                    loss += self.backward(&pass_out, &pass_tar, rate, &loss_fn);
                    pass_out = Vec::with_capacity(self.batch_size);
                    pass_tar = Vec::with_capacity(self.batch_size);
                }
            }
            if run(epoch, loss) {
                break;
            }
            epoch += 1;
            loss = 0.0;            
        }
 
        // remove the tracers from the layers before finishing
        self.layers
            .iter_mut()
            .for_each(|x| x.layer.remove_tracer());

        Ok(())
    }

    

    /// backpropagate the network, will move through time if needed
    #[inline]
    pub fn backward(&mut self, net_outs: &[Vec<f32>], net_targets: &[Vec<f32>], rate: f32, loss_fn: &Loss) -> f32 {
        let mut total_loss = 0.0;
        for i in (0..net_outs.len()).rev() {
            let errors = vectorops::loss(&net_targets[i], &net_outs[i], &loss_fn);
            total_loss += errors.0;
            self.layers
                .iter_mut()
                .rev()
                .fold(errors.1, |res, curr| {
                    curr.layer.backward(&res, rate).unwrap()
                });
        }
        self.reset();
        total_loss
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
        Some(data_transfer.to_owned())
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
    pub fn lstm(mut self, size: u32, output_size: u32, act: Activation) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(output_size).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::LSTM,
            layer: Box::new(LSTM::new(input_size, size, output_size, act))
        };
        self.layers.push(wrapper);
        self
    }



    #[inline]
    pub fn gru(mut self, size: u32, output_size: u32, act: Activation) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(output_size).unwrap();
        let wrapper = LayerWrap {
            layer_type: LayerType::GRU,
            layer: Box::new(GRU::new(input_size, size, output_size, act))
        };
        self.layers.push(wrapper);
        self
    }

    

    /// in order to more efficiently give inputs to the network, this function simple
    /// finds the shape of the layer that should be created based on the desired size
    #[inline]
    fn get_layer_sizes(&self, size: u32) -> Option<(u32, u32)> {
        if self.layers.len() == 0 {
            return Some((self.input_size, size))
        } 
        Some((self.layers.last()?.layer.shape().1 as u32, size))
    }


    
    /// dumy model saver file to export the model to json
    pub fn save(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer_pretty(&File::create(file_path)?, &self)?;
        Ok(())
    }



    /// load in a saved neat model from a file path
    pub fn load(file_path: &str) -> Result<Neat, Box<dyn Error>> {
        Ok(serde_json::from_reader(File::open(file_path).expect("file not found")).expect("error while reading json"))
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
            input_size: self.input_size,
            batch_size: self.batch_size
        }
    }
}

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



/// implement genome for a neat network
impl Genome<Neat, NeatEnvironment> for Neat {

    #[inline]
    fn crossover(one: &Neat, two: &Neat, env: Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<Neat> {
        let mut result_layers = Vec::with_capacity(one.layers.len());
        // iterate through the layers of the network and cross them over with each other
        for (one_layer, two_layer) in one.layers.iter().zip(two.layers.iter()) {
            let new_layer: Box<dyn Layer> = match one_layer.layer_type {
                LayerType::Dense | LayerType::DensePool => {
                    Box::new(Dense::crossover(one_layer.as_ref(), two_layer.as_ref(), Arc::clone(&env), crossover_rate)?)
                },
                LayerType::LSTM => {
                    Box::new(LSTM::crossover(one_layer.as_ref(), two_layer.as_ref(), Arc::clone(&env), crossover_rate)?)
                },
                LayerType::GRU => {
                    Box::new(GRU::crossover(one_layer.as_ref(), two_layer.as_ref(), Arc::clone(&env), crossover_rate)?)
                }
            };

            result_layers.push(LayerWrap {
                layer_type: one_layer.layer_type,
                layer: new_layer
            });
        }
        // return the new child network
        Some(Neat { 
            layers: result_layers, 
            input_size: one.input_size, 
            batch_size: one.batch_size
        })
    }



    fn base(env: &mut NeatEnvironment) -> Neat {
        Neat::new().input_size(env.input_size.unwrap()).dense_pool(env.output_size.unwrap(), Activation::Sigmoid)
    }


    #[inline]
    fn distance(one: &Neat, two: &Neat, env: Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut total_distance = 0.0;
        for (layer_one, layer_two) in one.layers.iter().zip(two.layers.iter()) {
            total_distance += match layer_one.layer_type {
                LayerType::Dense | LayerType::DensePool => {
                    Dense::distance(layer_one.as_ref(), layer_two.as_ref(), Arc::clone(&env))
                },
                LayerType::LSTM => {
                    LSTM::distance(layer_one.as_ref(), layer_two.as_ref(), Arc::clone(&env))
                },
                LayerType::GRU => {
                    GRU::distance(layer_one.as_ref(), layer_two.as_ref(), Arc::clone(&env))
                }
            };
        }
        total_distance
    }

}
