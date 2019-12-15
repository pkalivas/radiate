// pub mod neuron;
pub mod neat;
pub mod edge;
pub mod neatenv;
pub mod neuron;
pub mod layers;



pub mod activation {
        
    use std::f64::consts::E as Eul;

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


    impl Activation {

        /// Generic activation functions for an neural network - note for a few
        /// of these an alpha parameter is needed when first assining the function
        #[inline]
        pub fn activate(&self, x: f64) -> f64 {
            match self {
                Self::Sigmoid => {
                    1.0 / (1.0 + (-x * 4.9).exp())
                },
                Self::Tahn => {
                    x.tanh()
                },
                Self::Relu => { 
                    if x > 0.0 { 
                        x 
                    } else { 
                        0.0 
                    }
                },
                Self::Linear(alpha) => {
                    alpha * x
                },
                Self::LeakyRelu(alpha) => {
                    let a = alpha * x;
                    if a > x {
                        return a;
                    } 
                    x
                },
                Self::ExpRelu(alpha) => {
                    if x >= 0.0 {
                        return x;
                    }
                    alpha * (Eul.powf(x) - 1.0)
                }
            }
        }



        /// Deactivation functions for the activation neurons 
        #[inline]
        pub fn deactivate(&self, x: f64) -> f64 {
            match self {
                Self::Sigmoid => {
                    x * (1.0 - x)
                },
                Self::Tahn => {
                    1.0 - (x).powf(2.0)
                },
                Self::Linear(alpha) => {
                    *alpha
                },
                Self::Relu => {
                    if x > 0.0 { 
                        return 1.0;
                    }
                    0.0
                },
                Self::ExpRelu(alpha) => {
                    if x > 0.0 {
                        return 1.0;
                    }
                    alpha * x.exp()
                },
                Self::LeakyRelu(alpha) => {
                    if x > 0.0 { 
                        return 1.0;
                    } 
                    *alpha 
                },
            }
        }
    }
}



/// A neural network is made up of an input layer, hidden layers, and an output layer
pub mod neurontype {

    /// Because NEAT isn't exactly a traditional neural network there are no 'layers'.
    /// However there does need to be input nodes, hidden nodes, and output nodes.
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum NeuronType {
        Input,
        Output,
        Hidden,
    }

}



/// keep track of innovation numbers for neat 
/// this thing doesn't deserve it's own file its too small
pub mod counter {
        
    #[derive(Debug, Clone)]
    pub struct Counter {
        num: i32
    }

    impl Counter {
        pub fn new() -> Self {
            Counter {
                num: 0
            }
        }

        pub fn next(&mut self) -> i32 {
            let result = self.num;
            self.num += 1;
            result
        }

        pub fn roll_back(&mut self, num: i32) {
            self.num -= num;
        }
    }
}