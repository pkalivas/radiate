pub mod range;
pub mod sequence;

pub use range::RangeLookup;
pub use sequence::BoundedFixedSequence;

pub trait Sequence<T> {
    fn get(&self, index: usize) -> Option<&T>;
    fn get_mut(&mut self, index: usize) -> Option<&mut T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}
