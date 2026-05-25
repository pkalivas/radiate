//! Workspace-shared test helpers and fixtures for Radiate.
//!
//! Used as a `dev-dependency` by other crates in the workspace —
//! `radiate-engines`, `radiate-selectors`, etc. — so test scaffolding
//! is consistent across crates without each one duplicating its own
//! `tests/common/` directory.
//!
//! The crate is `publish = false` and never ships to crates.io.
//!
//! ## What's here
//!
//! - [`seeded`] — shorthand for `random_provider::scoped_seed`.
//! - [`MockEcosystem`] / [`MockEcosystemBuilder`] — declarative
//!   ecosystem and population builders with optional species and
//!   configurable scoring.
//! - [`mock_recombine_step`], [`mock_speciate_step`] — pre-wired step
//!   constructors for step-level unit tests.
//! - Stock alters per chromosome family: [`default_float_alters`],
//!   [`default_int_alters`], [`default_bit_alters`].
//! - Reusable problems and engine builders: [`OneMax`], [`Sphere`],
//!   [`sphere_engine`], [`onemax_engine`], etc.
//! - Convergence/invariant assertions: [`assert_within_budget`],
//!   [`assert_population_integrity`], etc.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod assertions;
pub mod fixtures;
pub mod mock;
pub mod problems;

pub use assertions::*;
pub use fixtures::*;
pub use mock::*;
pub use problems::*;

use radiate_core::random_provider;

/// Run `body` with a deterministic RNG seed. Wraps
/// `random_provider::scoped_seed`, which swaps the thread-local RNG
/// for the closure's duration so test isolation works under cargo's
/// parallel test runner.
pub fn seeded<R>(seed: u64, body: impl FnOnce() -> R) -> R {
    random_provider::scoped_seed(seed, body)
}
