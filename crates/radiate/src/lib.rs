//! A Rust library for genetic algorithms and artificial evolution.
//!
//! This crate provides a comprehensive set of tools for implementing genetic algorithms
//! and evolutionary computation in Rust. It includes core traits, selection strategies,
//! and genetic operators.

pub use radiate_engines::*;

#[cfg(feature = "gp")]
pub use radiate_gp::*;
