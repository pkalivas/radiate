mod error;
mod indices;
mod shape;
mod tensor;

pub use error::TensorError;
#[allow(dead_code)]
pub use shape::{Shape, Strides};
#[allow(dead_code)]
pub use tensor::Tensor;
