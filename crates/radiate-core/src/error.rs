pub use radiate_error::*;

pub type RadiateResult<T> = std::result::Result<T, RadiateError>;
