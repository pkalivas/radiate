extern crate simple_matrix;

use std::f32::consts::E as Eul;
use rand::Rng;
use rand::rngs::ThreadRng;
use simple_matrix::Matrix;



/// The neural network struct is meant to represent a simple feed forward neural network
/// with the ability to maniupate it's weights and biases, and feed forward a 1D vector
/// of a predetermined size (input_size). 
#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    pub weights: Vec<Matrix<f32>>,
    pub biases: Vec<Matrix<f32>>,
    input_size: i32,
}



/// implement the nerual network
impl NeuralNetwork {
    
    /// Create a new neural network
    /// 
    /// This returns a completely empty neural network, 
    /// to create a random network chain the 'fill_random()'
    /// method ontop of this method call.
    pub fn new(input_size: i32) -> Self {
        NeuralNetwork {
            weights: Vec::new(),
            biases: Vec::new(),
            input_size,
        }
    }



    /// Randomly fill the weights and biases of the neural network
    /// This method consumes the network, generates random vecs representing the 
    /// weights and biases, assigns them, and returns the consumed NeuralNetwork
    /// back to the caller.
    #[inline]    
    pub fn fill_random(mut self) -> Self {
        let (weights, biases) = self.generate_random_network();
        self.weights = weights;
        self.biases = biases;
        self
    }



    /// Edit the weights randomly of the matrix objects within the network
    /// pub fn edit_weights(&mut self, layer_rate: f32, weight_mutate: f32, weight_transform: f32) {
    #[inline]
    pub fn edit_weights(&mut self, weight_mutate: f32, weight_transform: f32, layer_mutate: f32) {
        // create a closure to apply to each the weights and the biases
        // which randomly transforms the given weight be a given weight transform amount 
        // or uniformly changed.
        // need to create a new rand thread_rng because during concurrent weight editing, there is a change
        // the network's self random will try to generage both an f32 and an f32 at the same time
        // resulting in a program wide panic! that unwinds the stack.
        let mut temp_rand = rand::thread_rng();
        let transform = |x: &mut f32| {
            let mut r = rand::thread_rng();
            if r.gen::<f32>() < weight_mutate {
                *x *= r.gen_range(-weight_transform, weight_transform);
            } else {
                *x = r.gen::<f32>();
            }
        };

        // iterate through the weights and the layers and apply the function
        for (weight, bias) in self.weights.iter_mut().zip(self.biases.iter_mut()) {
            if temp_rand.gen::<f32>() < layer_mutate {
                weight.apply_mut(transform);
                bias.apply_mut(transform);
            }
        }
    }



    /// Generate a random neural network with at least one hidden layer and each layer with a size
    /// of at least one. Return a tuble containin the vec of weights represented by a simple 
    /// matrix and a vec of biases represeted by a simple matrix as well.
    #[inline]    
    pub fn generate_random_network(&mut self) -> (Vec<Matrix<f32>>, Vec<Matrix<f32>>) {
        // initialize the vecs and keep track of the previous size so the matrix mutiplication
        // matches correctly https://www.mathsisfun.com/algebra/matrix-multiplying.html
        // Then create a list of layer sizes in range (1, 4], with sizes (1, 32]
        let mut r = rand::thread_rng();
        let (mut weights, mut biases) = (Vec::new(), Vec::new());
        let mut previous_size = self.input_size as usize;
        let sizes = (0..r.gen_range(1, 4))
            .map(|_| r.gen_range(1, 32))
            .collect::<Vec<_>>();
        
        // loop through each layer size to create a network layer, keep the size of the last layer
        for layer in sizes {

            // get a vector of randomly generated f32 values with size layer * previous_size
            // then create a matrix out of each returned value 
            let (weight_data, biase_data) = self.rand_layer_nums(layer, previous_size, &mut r);
            let curr_weight = Matrix::from_iter(layer, previous_size, weight_data);
            let curr_bias = Matrix::from_iter(layer, 1, biase_data);

            // add the matrices to the network then transfer the previous size
            weights.push(curr_weight);
            biases.push(curr_bias);
            previous_size = layer;
        }
        
        // get the random values for the output layer of the neural net, this is nessecary because 
        // the output will be of size 2, so we must make sure the network matches that shape
        let (weight_data, biase_data) = self.rand_layer_nums(2, previous_size, &mut r);
        weights.push(Matrix::from_iter(2, previous_size, weight_data));
        biases.push(Matrix::from_iter(2, 1, biase_data));

        // return the weights and biases
        (weights, biases)
    }



    /// feed forward a matrix through the neural network and output a matrix<f32>
    /// if the input shape does not fit matrix multiplication rules, method will Panic!
    /// Note: the input matrix must already by transmuted to where the input rows 
    /// should equal the first layer's column -> dot product
    #[inline]
    pub fn feed_forward(&self, mut input: Matrix<f32>) -> Matrix<f32> {
        for (weight, bias) in self.weights.iter().zip(self.biases.iter()) {
            let mut layer_output = &(weight * &input) + bias;
            layer_output.apply_mut(|x| *x = NeuralNetwork::sigmoid(x));
            input = layer_output;
        }
        input
    }



    /// Create two lists with randomly generated f32 values represetnting the weights
    /// and biases of the neural network. Return them in a tuple
    #[inline]    
    fn rand_layer_nums(&mut self, rows: usize, cols: usize, r: &mut ThreadRng) -> (Vec<f32>, Vec<f32>) {
        (
            (0..(rows * cols))
                .map(|_| r.gen::<f32>())
                .collect::<Vec<_>>(),
            (0..rows)
                .map(|_| 1.0)
                .collect::<Vec<_>>()
        )
    }



    /// Compute the sum of the weights of the neural network
    /// pretty simple.
    #[inline]
    pub fn weight_sum(&self) -> f32 {
        let mut total: f32 = 0.0;
        for weight in self.weights.iter() {
            weight.apply(|x| total += *x);
        }
        total
    }



    /// Sigmoid function for as an activation function for the nerual network between layers
    #[allow(dead_code)]
    fn sigmoid(x: &f32) -> f32 {
        1.0 / (1.0 + Eul.powf(*x * -1.0))
    }


}





/// Override the partialeq trait for the neural network. This is needed because there is 
/// no impmenetation for partialeq for a ThreadRng which the nerual net ownes. Because 
/// that doesn't matter for a partialeq, this override only checks what is actually needed. 
/// Without this the program will not compile.
impl PartialEq for NeuralNetwork {
    fn eq(&self, other: &Self) -> bool {
        self.weights == other.weights && self.biases == other.biases
    }
}



impl Drop for NeuralNetwork {
    fn drop(&mut self) {}
}



// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_feed_forward() {
//         let input = vec![1.0, 0.5, 0.7, 1.0];
//         let input_matrix = Matrix::from_iter(input.len(), 1, input);
//         let network = NeuralNetwork::new(4).fill_random();
//         let output = network.feed_forward(input_matrix);
//         assert!(output.to_vec().len() == 2, true);
//     }
// }