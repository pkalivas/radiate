mod error;
mod indices;
mod matrix;
mod shape;
mod tensor;

pub use error::TensorError;
pub use matrix::Matrix;
#[allow(dead_code)]
pub use shape::{Shape, Strides};
#[allow(dead_code)]
pub use tensor::Tensor;
