mod genome;
mod temporal;
mod value;

pub use genome::{AnyChromosome, AnyGene};
pub use temporal::*;
pub use value::{
    AnyValue, DataType, Field, Scalar, apply_zipped_slice, apply_zipped_struct_slice, dtype_names,
    mean_anyvalue,
};
