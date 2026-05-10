//! Reusable benchmark problems. Each one is small, fast-converging, and
//! has a known optimum so tests can assert reaching it within a budget.
//!
//! These are deliberately tiny — the goal is to exercise the engine
//! plumbing, not to be a serious benchmark suite. If a future test
//! needs a harder problem, add it here so other tests can share it.

#![allow(dead_code)]

use radiate_core::*;

/// One-Max: maximize the count of `true` bits in a fixed-length bitstring.
/// Optimum is `n_bits`. Trivial to solve, useful for sanity-checking the
/// bit-codec pathway and selector × alterer matrices.
pub struct OneMax {
    pub n_bits: usize,
}

impl OneMax {
    pub const fn new(n_bits: usize) -> Self {
        Self { n_bits }
    }

    pub fn fitness_fn(&self) -> impl Fn(Vec<bool>) -> i32 + Send + Sync + 'static {
        |geno: Vec<bool>| geno.iter().filter(|b| **b).count() as i32
    }

    pub fn codec(&self) -> BitCodec<Vec<bool>> {
        BitCodec::vector(self.n_bits)
    }

    /// The optimum score (all bits set).
    pub fn optimum(&self) -> i32 {
        self.n_bits as i32
    }
}

/// Int-minimize-to-zero: minimize the sum of integer chromosome values
/// drawn from `0..max`. Optimum is 0 (all genes at zero).
pub struct IntMinimizeToZero {
    pub n_genes: usize,
    pub max: i32,
}

impl IntMinimizeToZero {
    pub const fn new(n_genes: usize, max: i32) -> Self {
        Self { n_genes, max }
    }

    pub fn codec(&self) -> IntCodec<i32, Vec<i32>> {
        IntCodec::vector(self.n_genes, 0..self.max)
    }

    pub fn fitness_fn(&self) -> impl Fn(Vec<i32>) -> i32 + Send + Sync + 'static {
        |geno: Vec<i32>| geno.iter().sum::<i32>()
    }

    pub fn optimum(&self) -> i32 {
        0
    }
}

/// Quadratic regression: fit `a*x^2 + b*x + c` to a dense set of `(x, y)`
/// samples drawn from `y = x^2` over `[-2, 2]`. Optimum is MSE → 0
/// (achieved by the coefficients `a=1, b=0, c=0`).
///
/// Useful for exercising the float pathway: real-valued chromosomes,
/// blend crossover, gaussian mutation, etc.
pub struct QuadraticRegression {
    pub n_samples: usize,
}

impl QuadraticRegression {
    pub const fn new(n_samples: usize) -> Self {
        Self { n_samples }
    }

    pub fn codec(&self) -> FloatCodec<f32, Vec<f32>> {
        FloatCodec::vector(3, -5.0..5.0)
    }

    /// Returns a closure that computes MSE for a candidate `(a, b, c)`.
    pub fn fitness_fn(&self) -> impl Fn(Vec<f32>) -> f32 + Send + Sync + 'static {
        let n = self.n_samples;
        let test_inputs: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1).max(1) as f32; // [0, 1]
                -2.0 + 4.0 * t // map to [-2, 2]
            })
            .collect();
        let targets: Vec<f32> = test_inputs.iter().map(|x| x * x).collect();

        move |coeffs: Vec<f32>| -> f32 {
            test_inputs
                .iter()
                .zip(targets.iter())
                .map(|(x, y)| {
                    let pred = coeffs[0] * x * x + coeffs[1] * x + coeffs[2];
                    (pred - y).powi(2)
                })
                .sum::<f32>()
                / test_inputs.len() as f32
        }
    }
}

/// Sphere function: minimize sum of squares of float genes. Smooth,
/// convex, continuously differentiable. The "always converges fast"
/// problem — useful for stability/long-run tests where you want the
/// engine to find the optimum but you mainly care about non-pathology.
pub struct Sphere {
    pub n_genes: usize,
    pub range: f32,
}

impl Sphere {
    pub const fn new(n_genes: usize, range: f32) -> Self {
        Self { n_genes, range }
    }

    pub fn codec(&self) -> FloatCodec<f32, Vec<f32>> {
        FloatCodec::vector(self.n_genes, -self.range..self.range)
    }

    pub fn fitness_fn(&self) -> impl Fn(Vec<f32>) -> f32 + Send + Sync + 'static {
        |geno: Vec<f32>| geno.iter().map(|x| x * x).sum::<f32>()
    }
}
