mod sorted;
mod value;
mod window;

use smallvec::SmallVec;
pub use sorted::SortedBuffer;
pub use value::Value;
pub use window::WindowBuffer;

pub enum Inner<T> {
    Small(SmallVec<[T; 4]>),
    Large(Vec<T>),
}

pub struct Store<T> {
    inner: Inner<T>,
}
