pub mod neat;
pub mod edge;
pub mod neatenv;
pub mod neuron;
pub mod layers;
pub mod tracer;



pub mod activation {


    use std::f32::consts::E as Eul;

    /// Varius activation functions for a neuron, must be specified at creation
    #[derive(Debug, PartialEq, Clone, Copy)]
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
    #[derive(Debug, PartialEq, Clone, Copy)]
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

    use super::neat::Neat;
    use super::tracer::Tracer;
    use super::activation::Activation;
    use super::neurontype::NeuronType;
    use super::layers::layertype::LayerType;
    use super::layers::dense::Dense;
    use super::layers::lstm::LSTM;
    use super::layers::lstm::LSTMState;
    use super::edge::Edge;
    use super::neuron::Neuron;



    impl Serialize for Activation {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("Activation", 1)?;
            match self {
                Self::Sigmoid => s.serialize_field("sigmoid", &1)?,
                Self::Tahn => s.serialize_field("tahn", &1)?,
                Self::Relu => s.serialize_field("tahn", &1)?,
                Self::Softmax => s.serialize_field("softmax", &1)?,
                Self::Linear(x) => s.serialize_field("linear", &x)?,
                Self::LeakyRelu(x) => s.serialize_field("leakyrelu", &x)?,
                Self::ExpRelu(x) => s.serialize_field("exprelu", &x)?,
            }
            s.end()
        }
    }



    impl Serialize for NeuronType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("NeuronType", 1)?;
            match self {
                Self::Input => s.serialize_field("type", &"Input")?,
                Self::Hidden => s.serialize_field("type", &"Hidden")?,
                Self::Output => s.serialize_field("type", &"Output")?,
            }
            s.end()
        }
    }



    impl Serialize for LayerType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("LayerType", 1)?;
            s.serialize_field("layer_type", &1)?;
            s.end()
        }
    }



        
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


    impl Serialize for Tracer {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("Tracer", 4)?;
            s.serialize_field("neuron_activation", &self.neuron_activation)?;
            s.serialize_field("neuron_derivative", &self.neuron_derivative)?;
            s.serialize_field("max_neuron_index", &self.max_neuron_index)?;
            s.serialize_field("index", &self.index)?;
            s.end()
        }
    }

    
        
    impl Serialize for LSTMState {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("LSTMState", 7)?;
            s.serialize_field("f_gate_output", &self.f_gate_output)?;
            s.serialize_field("i_gate_output", &self.i_gate_output)?;
            s.serialize_field("s_gate_output", &self.s_gate_output)?;
            s.serialize_field("o_gate_output", &self.o_gate_output)?;
            s.serialize_field("memory_states", &self.memory_states)?;
            s.serialize_field("d_prev_memory", &self.d_prev_memory)?;
            s.serialize_field("d_prev_hidden", &self.d_prev_hidden)?;
            s.end()
        }
    }


    impl Serialize for LSTM {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("LSTMState", 11)?;
            s.serialize_field("input_size", &self.input_size)?;
            s.serialize_field("memory_size", &self.memory_size)?;
            s.serialize_field("output_size", &self.output_size)?;
            s.serialize_field("memory", &self.memory)?;
            s.serialize_field("hidden", &self.hidden)?;
            s.serialize_field("states", &self.states)?;
            s.serialize_field("g_gate", &self.g_gate)?;
            s.serialize_field("i_gate", &self.i_gate)?;
            s.serialize_field("f_gate", &self.f_gate)?;
            s.serialize_field("o_gate", &self.o_gate)?;
            s.serialize_field("v_gate", &self.v_gate)?;
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
                .collect::<Vec<_>>();
            s.serialize_field("inputs", &self.inputs)?;
            s.serialize_field("outputs", &self.outputs)?;
            s.serialize_field("nodes", &n)?;
            s.serialize_field("edges", &self.edges)?;
            s.serialize_field("layer_type", &self.layer_type)?;
            s.serialize_field("activation", &self.activation)?;
            s.end()
        }
    }



           
    impl Serialize for Edge {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("Edge", 5)?;
            s.serialize_field("src", &self.src)?;
            s.serialize_field("dst", &self.dst)?;
            s.serialize_field("innov", &self.innov)?;
            s.serialize_field("weight", &self.weight)?;
            s.serialize_field("active", &self.active)?;
            s.end()
        }
    }




    impl Serialize for Neuron {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut s = serializer.serialize_struct("Neuron", 10)?;
            s.serialize_field("innov", &self.innov)?;
            s.serialize_field("bias", &self.bias)?;
            s.serialize_field("value", &self.value)?;
            s.serialize_field("d_value", &self.d_value)?;
            s.serialize_field("error", &self.error)?;
            s.serialize_field("state", &self.state)?;
            s.serialize_field("outgoing", &self.outgoing)?;
            s.serialize_field("incoming", &self.incoming)?;
            s.serialize_field("activation", &self.activation)?;
            s.serialize_field("neuron_type", &self.neuron_type)?;
            s.end()
        }
    }

    

}
