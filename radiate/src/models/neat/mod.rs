pub mod neat;
pub mod edge;
pub mod neatenv;
pub mod neuron;
pub mod layers;
pub mod tracer;



pub mod loss {

    pub enum Loss {
        MSE,
        Diff
    }
}


pub mod activation {

    use std::f32::consts::E as Eul;

    /// Varius activation functions for a neuron, must be specified at creation
    #[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
    pub enum Activation {
        Sigmoid,
        Tahn,
        Relu,
        Softmax,
        LeakyRelu(f32),
        ExpRelu(f32),
        Linear(f32)   
    }


    impl Activation {

        /// Generic activation functions for an neural network - note for a few
        /// of these an alpha parameter is needed when first assining the function
        #[inline]
        pub fn activate(&self, x: f32) -> f32 {
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
                },
                _ => panic!("Cannot activate single neuron")

            }
        }



        /// Deactivation functions for the activation neurons 
        #[inline]
        pub fn deactivate(&self, x: f32) -> f32 {
            match self {
                Self::Sigmoid => {
                    self.activate(x) * (1.0 - self.activate(x))
                },
                Self::Tahn => {
                    1.0 - (self.activate(x)).powf(2.0)
                },
                Self::Linear(alpha) => {
                    *alpha
                },
                Self::Relu => {
                    if self.activate(x) > 0.0 { 
                        return 1.0;
                    }
                    0.0
                },
                Self::ExpRelu(alpha) => {
                    if self.activate(x) > 0.0 {
                        return 1.0;
                    }
                    alpha * x.exp()
                },
                Self::LeakyRelu(alpha) => {
                    if self.activate(x) > 0.0 { 
                        return 1.0;
                    } 
                    *alpha 
                },
                _ => panic!("Cannot deactivate single neuron")
            }
        }
    }
}


/// A neural network is made up of an input layer, hidden layers, and an output layer
pub mod neurontype {

    
    /// Because NEAT isn't exactly a traditional neural network there are no 'layers'.
    /// However there does need to be input nodes, hidden nodes, and output nodes.
    #[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
    pub enum NeuronType {
        Input,
        Output,
        Hidden
    }

}
