extern crate rand;

use std::sync::{Arc, RwLock};
use rand::Rng;
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



/// Neat is a neural network consisting of layers
/// the layers can be stacked together then the feed forward
/// and backprop functions will take care of 'connecting them'
#[derive(Debug)]
pub struct Neat {
    pub layers: Vec<Box<dyn Layer>>
}



impl Neat {

    
    pub fn new() -> Self {
        Neat {
            layers: Vec::new()
        }
    }

    

    /// feed forward a vec of data through the neat network 
    #[inline]
    pub fn feed_forward(&mut self, data: &Vec<f64>) -> Option<Vec<f64>> {
        // keep two vec in order to transfer the data from one layer to another layer in the network
        let mut data_transfer = data;
        let mut temp;
        for layer in self.layers.iter_mut() {
            // println!("INPUT: {:?}", data_transfer);
            temp = layer.propagate(data_transfer)?;
            // println!("OUTPUT: {:?}", temp);
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
        let feed_out = self.feed_forward(data).unwrap();
        // compute the original errors
        let errors = &output.iter().zip(feed_out.iter())
            .map(|(target, prediction)| target - prediction)
            .collect::<Vec<_>>();
        
        // println!("INPUT: {:?}", data);
        // println!("TARGET: {:?}", output);
        // println!("OUTPUT: {:?}", feed_out);
        // println!("ERRORS: {:?}", errors);
        // similar to feed_forward, keep mutable vecs in order to transfer the error
        // from the one layer back to the layer preceding it
        let mut temp;
        let mut data_transfer = errors;
        for layer in self.layers.iter_mut().rev() {
            temp = layer.backprop(data_transfer, learning_rate).unwrap();
            // println!("TEMP: {:?}", temp);
            data_transfer = &temp;
        }
        // println!("\n\n\n");
    }



    /// create and append a new dense pool layer onto the neat network
    #[inline]
    pub fn dense_pool(mut self, size: i32, env: &mut NeatEnvironment) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(size, env).unwrap();
        self.layers.push(Box::new(Dense::new(input_size, output_size, LayerType::DensePool, Activation::Sigmoid, env.get_mut_counter())));
        self
    }


    /// create an append a simple dense layer onto the network
    #[inline]
    pub fn dense(mut self, size: i32, env: &mut NeatEnvironment) -> Self {
        let (input_size, output_size) = self.get_layer_sizes(size, env).unwrap();
        self.layers.push(Box::new(Dense::new(input_size, output_size, LayerType::Dense, Activation::Sigmoid, env.get_mut_counter())));
        self
    }

    

    /// in order to more efficently give inputs to the network, this function simple 
    /// finds the shape of the layer that should be created based on the desired size
    #[inline]
    fn get_layer_sizes(&self, size: i32, env: &mut NeatEnvironment) -> Option<(i32, i32)> {
        if self.layers.len() == 0 {
            return Some((env.input_size?, size))
        } 
        Some((self.layers.last()?.shape().1 as i32, size))
    }



}


/// Implement clone for the neat neural network in order to facilitate 
/// proper crossover and mutation for the network
impl Clone for Neat {
    fn clone(&self) -> Self {
        Neat {
            layers: self.layers
                .iter()
                .map(|x| x.clone())
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

        true
    }
}




/// iplement genome for a neat network
impl Genome<Neat, NeatEnvironment> for Neat {

    #[inline]
    fn crossover(one: &Neat, two: &Neat, env: &Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<Neat> {
        let mut set = (*env).write().ok()?;
        let mut r = rand::thread_rng();
        let mut result = (*one).clone();

        unsafe {
            // if a random number is less than the cross rate, then just crossover the 
            // topological strucutre of the networks to create a child network 
            if r.gen::<f32>() < crossover_rate {
                // for (innov, edge) in result.edges.iter_mut() {
                //     // if the edge is in both networks, then radnomly assign the weight to the edge
                //     if two.edges.contains_key(innov) {
                //         if r.gen::<f32>() < 0.5 {
                //             edge.weight = two.edges.get(innov)?.weight;
                //         }
                //         // if the edge is deactivated in either network and a random number is less than the 
                //         // reactivate parameter, then reactiveate the edge and insert it back into the network
                //         if (!edge.active || !two.edges.get(innov)?.active) && r.gen::<f32>() < set.reactivate? {
                //             (**result.nodes.get(&edge.src)?).outgoing.push(*innov);
                //             (**result.nodes.get(&edge.dst)?).incoming.insert(*innov, None);
                //             edge.active = true;
                //         }
                //     }
                // }
            } else {
                // if a random number is less than the edit_weights parameter, then edit the weights of the network edges
                // add a possible new node to the network randomly 
                // attempt to add a new edge to the network, there is a chance this operation will add no edge
                // if r.gen::<f32>() < set.weight_mutate_rate? {
                //     result.edit_weights(set.edit_weights?, set.weight_perturb?);
                // }
                // if r.gen::<f32>() < set.new_node_rate? {
                //     let act_func = *set.activation_functions.choose(&mut r)?;
                //     let node_type = *set.node_types.choose(&mut r)?;
                //     let new_node = result.add_node(set.get_mut_counter(), node_type, act_func)?;
                //     Neat::neuron_control(&mut result, &new_node, &mut set).ok()?;
                // }
                // if r.gen::<f32>() < set.new_edge_rate? {
                //     let new_edge = result.add_edge(set.get_mut_counter());
                //     Neat::edge_control(&mut result, new_edge, &mut set);
                // }
            }
        }
        Some(result)
    }


    fn base(env: &mut NeatEnvironment) -> Neat {
        Neat::new()
    }


    fn distance(one: &Neat, two: &Neat, env: &Arc<RwLock<NeatEnvironment>>) -> f64 {
        // keep track of the number of excess and disjoint genes and the
        // average weight of shared genes between the two networks 
        let (mut e, mut d) = (0.0, 0.0);
        let (mut w, mut wc) = (0.0, 0.0);
        // determin the largest network and it's max innovation number
        // and store that and the smaller network and it's max innovation number
        // let one_max = one.max_marker();
        // let two_max = two.max_marker();
        // let (big, small, small_innov) = if one_max > two_max { 
        //     (one, two, two_max)
        // } else { 
        //     (two, one, one_max)
        // };
        // // iterate through the larger network 
        // for (innov, edge) in big.edges.iter() {
        //     // check if it's a sharred innvation number
        //     if small.edges.contains_key(innov) {
        //         w += (edge.weight - small.edges.get(innov).unwrap().weight).abs();
        //         wc += 1.0;
        //         continue;
        //     }
        //     if innov > &small_innov {
        //         e += 1.0;
        //     } else {
        //         d += 1.0;
        //     }
        // }
        // // disjoint genes can be found within both networks unlike excess, so we still need to 
        // // go through the smaller network and see if there are any disjoint genes in there as well
        // for innov in small.edges.keys() {
        //     if !big.edges.contains_key(innov) {
        //         d += 1.0;
        //     }
        // }
        // // lock the env to get the comparing values from it  and make sure wc is greater than 0
        // let wc = if wc == 0.0 { 1.0 } else { wc };
        // let lock_env = (*env).read().unwrap();
        // // return the distance between the two networks
        // ((lock_env.c1.unwrap() * e) / big.edges.len() as f64) + ((lock_env.c2.unwrap() * d) / big.edges.len() as f64) + (lock_env.c3.unwrap() * (w / wc))
        0.0
    }

}
