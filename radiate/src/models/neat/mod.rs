pub mod neuron;
pub mod neat;
pub mod edge;
pub mod neatenv;
pub mod activation;
pub mod nodetype;
pub mod neur;
pub mod dense;

/// A neural network is made up of an input layer, hidden layers, and an output layer
pub mod layer {
    /// Because NEAT isn't exactly a traditional neural network there are no 'layers'.
    /// However there does need to be input nodes, hidden nodes, and output nodes.
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Layer {
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