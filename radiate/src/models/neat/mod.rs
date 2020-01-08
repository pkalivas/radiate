pub mod neat;
pub mod edge;
pub mod neatenv;
pub mod neuron;
pub mod layers;
pub mod tracer;



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




/// serialize and deserialize the neat model
pub mod inputoutput {

    use serde::ser::{Serialize, SerializeStruct, Serializer};
    use serde_json::json;
    use std::collections::HashMap;

    use super::neat::Neat;
    use super::layers::layertype::LayerType;
    use super::layers::dense::Dense;
    use super::layers::lstm::LSTM;


        
    impl Serialize for Neat {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("Neat", 2)?;
            let l = self.layers
            .iter()
            .map(|x| {
                match x.layer_type {
                    LayerType::Dense | LayerType::DensePool => {
                        json!{(&x.as_ref::<Dense>())}
                    },
                    LayerType::LSTM => {
                        json!{(&x.as_ref::<LSTM>())}
                    }
                }
            })
            .collect::<Vec<_>>();
            s.serialize_field("input_size", &self.input_size)?;
            s.serialize_field("layers", &l)?;
            s.end()
        }
    }


        
    impl Serialize for Dense {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("Dense", 7)?;
            let n = self.nodes
                .iter()
                .map(|x| (x.0, unsafe { (**x.1).clone() }) )
                .collect::<HashMap<_, _>>();
            s.serialize_field("inputs", &self.inputs)?;
            s.serialize_field("outputs", &self.outputs)?;
            s.serialize_field("nodes", &n)?;
            s.serialize_field("edges", &self.edges)?;
            s.serialize_field("layer_type", &self.layer_type)?;
            s.serialize_field("activation", &self.activation)?;
            s.end()
        }
    }

    

}
