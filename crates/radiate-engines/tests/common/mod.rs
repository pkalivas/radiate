//! Shared helpers for the integration test suite.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod assertions;
pub mod fixtures;
pub mod problems;
pub mod testing;

pub use assertions::*;
pub use fixtures::*;
pub use problems::*;
pub use testing::*;

use radiate_core::random_provider;

/// Tiny shorthand for `random_provider::scoped_seed`.
///
/// The engine draws RNG from a thread-local; `scoped_seed` swaps it for
/// the closure's duration and restores after, so test isolation works
/// even when cargo runs tests in parallel.
///
/// Usage:
/// ```ignore
/// seeded(42, || {
///     let engine = ...;
///     assert!(...);
/// });
/// ```
pub fn seeded<R>(seed: u64, body: impl FnOnce() -> R) -> R {
    random_provider::scoped_seed(seed, body)
}
