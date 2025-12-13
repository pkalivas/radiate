mod alters;
mod diversity;
mod executors;
mod limits;
mod selectors;

/// A trait for transforming Python input types into Radiate internal types.
/// This is used to convert various configurations provided in Python
/// into the corresponding Rust types that Radiate uses internally.
///
/// This is a somewhat complicated process due to the need to handle
/// different generic chromosome types and configurations, so this trait provides
/// a unified interface for performing these conversions.
pub trait InputTransform<O> {
    fn transform(&self) -> O;
}
