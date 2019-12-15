
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::fmt::{
    Debug,
    Formatter,
    Result
};
use super::super::neatenv::NeatEnvironment;



pub trait Mutate<L> 
    where L: Layer
{
    fn mutate(child: &mut L, parent_one: &L, parent_two: &L, env: &Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) 
        where 
            Self: Sized + Send + Sync;
}



pub trait Layer: LayerClone + Any {

    fn propagate(&mut self, inputs: &Vec<f64>) -> Option<Vec<f64>>;

    fn backprop(&mut self, errors: &Vec<f64>, learning_rate: f64) -> Option<Vec<f64>>;

    fn as_ref_any(&self) -> &dyn Any;

    fn as_mut_any(&mut self) -> &mut dyn Any;

    fn shape(&self) -> (usize, usize);

}



pub trait LayerClone {

    fn clone_box(&self) -> Box<dyn Layer>;

}



impl<L> LayerClone for L
    where L: 'static + Layer + Clone 
{
    fn clone_box(&self) -> Box<dyn Layer> {
        Box::new((*self).clone())
    }
}



impl Clone for Box<dyn Layer> {
    fn clone(&self) -> Box<dyn Layer> {
        self.clone_box()
    }
}



impl Debug for dyn Layer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Layer")
    }
}



impl PartialEq for dyn Layer {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}