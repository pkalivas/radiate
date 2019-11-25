

pub trait Neue {

    
    /// Turn the neuron into a raw mutable pointer - this
    /// makes the data structure inherintly unsafe 
    fn as_mut_ptr(self) -> *mut Self
        where Self: Sized 
    {
        Box::into_raw(Box::new(self))
    }

    fn is_ready(&mut self) -> bool;

    fn reset(&mut self);



}