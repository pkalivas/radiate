pub use radiate_core::{RadiateError, error::RadiateResult, error::Result};
pub use radiate_engines::*;

#[cfg(feature = "gp")]
pub use radiate_gp::*;

pub mod prelude {
    pub use radiate_engines::*;

    #[cfg(feature = "gp")]
    pub use radiate_gp::*;
}
