//! Engine builder fixtures for the recurring problem shapes.
//!
//! These return *partially-configured* `GeneticEngineBuilder`s — they set
//! the codec/fitness/objective for a known problem but leave
//! population_size, selectors, alterers, etc. as builder defaults so
//! individual tests can override what they care about.

#![allow(dead_code)]

use super::problems::{IntMinimizeToZero, OneMax, QuadraticRegression, Sphere};
use radiate_alters::{BlendCrossover, GaussianMutator, UniformCrossover, UniformMutator};
use radiate_core::*;
use radiate_engines::*;

/// A standard one-max engine: bit codec, maximize, uniform alters.
/// `n_bits` chooses problem size; everything else takes builder defaults.
pub fn onemax_engine(n_bits: usize) -> GeneticEngine<BitChromosome, Vec<bool>> {
    let problem = OneMax::new(n_bits);
    GeneticEngine::builder()
        .problem(problem)
        .alter(alters![
            UniformCrossover::new(0.7),
            UniformMutator::new(0.05)
        ])
        .build()
}

/// Standard int-minimize-to-zero engine.
pub fn int_minimize_engine(
    n_genes: usize,
    max: i32,
) -> GeneticEngine<IntChromosome<i32>, Vec<i32>> {
    let problem = IntMinimizeToZero::new(n_genes, max);
    GeneticEngine::builder()
        .minimizing()
        .problem(problem)
        .build()
}

/// Standard float-regression engine for the y=x^2 problem.
pub fn quadratic_regression_engine(
    n_samples: usize,
) -> GeneticEngine<FloatChromosome<f32>, Vec<f32>> {
    let problem = QuadraticRegression::new(n_samples);
    GeneticEngine::builder()
        .minimizing()
        .problem(problem)
        .alter(alters![
            BlendCrossover::new(0.5, 0.5),
            GaussianMutator::new(0.1)
        ])
        .build()
}

/// Standard sphere engine. `n_genes` chooses dimensionality.
pub fn sphere_engine(n_genes: usize) -> GeneticEngine<FloatChromosome<f32>, Vec<f32>> {
    let problem = Sphere::new(n_genes, 10.0);
    GeneticEngine::builder()
        .minimizing()
        .problem(problem)
        .alter(alters![
            BlendCrossover::new(0.5, 0.5),
            GaussianMutator::new(0.05)
        ])
        .build()
}

pub fn speciated_sphere_engine(
    n_genes: usize,
    pop_size: usize,
    species_threshold: f32,
) -> GeneticEngine<FloatChromosome<f32>, Vec<f32>> {
    let problem = Sphere::new(n_genes, 10.0);
    GeneticEngine::builder()
        .problem(problem)
        .minimizing()
        .population_size(pop_size)
        .diversity(EuclideanDistance)
        .species_threshold(species_threshold)
        .alter(alters![
            BlendCrossover::new(0.5, 0.5),
            GaussianMutator::new(0.05)
        ])
        .build()
}
