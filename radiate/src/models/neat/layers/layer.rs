
use std::any::Any;
use std::fmt::Debug;



/// Layer is a layer in the neural network. In order for 
/// the network to be evolved, it must be able to be cloned which is where LayerClone
/// comes in - allowing the Box<dyn Layer> to be cloned without knowing which type it 
/// really is under the hood. Any allows for the underlying object to be downcast to a concrete type
pub trait Layer: LayerClone + Any + Debug {
    
    /// propagate an input vec through this layer. This is done differently 
    /// depending on the type of layer, just the same as backpropagation is.
    /// Return the output as a vec 
    fn propagate(&mut self, inputs: &Vec<f64>) -> Option<Vec<f64>>;

    /// Take the errors of the feed forward and backpropagate them through the network
    /// to adjust the weights of the connections between the neurons. Return the error 
    /// of the input neurons from this layer - needed to transfer error from layer to layer
    fn backprop(&mut self, errors: &Vec<f64>, learning_rate: f64) -> Option<Vec<f64>>;

    /// Get a reference to the underlying type without generics in order to downcast to a concrete type
    fn as_ref_any(&self) -> &dyn Any;

    /// Get a mutable reference to the underlying type without generics in order to downcast to a concrete type
    fn as_mut_any(&mut self) -> &mut dyn Any;

    /// Return the (input size, output size) of this layer - used to make specifying layer sizes easier 
    /// so the user only needs the say the size of the output, not the input. That would be too redundant.
    fn shape(&self) -> (usize, usize);

}



/// Turns out cloning a Box<dyn trait> is harder than it seems. 
/// This isn't meant to be implemented outside of this file and is only used
/// to clone the trait object 
pub trait LayerClone {
    fn clone_box(&self) -> Box<dyn Layer>;
}



/// Implement LayerClone for any type <L> that implements Layer 
/// and is also Clone
impl<L> LayerClone for L
    where L: 'static + Layer + Clone 
{
    fn clone_box(&self) -> Box<dyn Layer> {
        Box::new((*self).clone())
    }
}



/// required for the dyn layer to be Clone in order for 
/// LayerClone to work, so impelement Clone for any Layer
impl Clone for Box<dyn Layer> {
    fn clone(&self) -> Box<dyn Layer> {
        self.clone_box()
    }
}


/// Need to able to compare dyn layers (is there a better way to do this?)
impl PartialEq for dyn Layer {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}