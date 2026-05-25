#![allow(dead_code)]

use std::collections::HashSet;

use radiate_core::*;
use radiate_engines::*;

pub fn assert_has_species<C: Chromosome>(result: &Ecosystem<C>, label: &str) {
    let species = result
        .species()
        .unwrap_or_else(|| panic!("{label}: expected species, but got None"));
    assert!(
        !species.is_empty(),
        "{label}: expected at least one species, but got an empty list"
    );
}

pub fn assert_no_species<C: Chromosome>(result: &Ecosystem<C>, label: &str) {
    assert!(
        result.species().is_none(),
        "{label}: expected no species, but got Some with {} species",
        result.species().unwrap().len()
    );
}

pub fn assert_species_count<C: Chromosome>(result: &Ecosystem<C>, expected: usize, label: &str) {
    let species = result
        .species()
        .unwrap_or_else(|| panic!("{label}: expected species, but got None"));
    assert_eq!(
        species.len(),
        expected,
        "{label}: expected {expected} species, but got {}",
        species.len()
    );
}

pub fn assert_population_speciated<C: Chromosome>(result: &Ecosystem<C>, label: &str) {
    let empty_id = SpeciesId::EMPTY;
    let pop = result.population();
    let species = result
        .species()
        .unwrap_or_else(|| panic!("{label}: expected species, but got None"))
        .iter()
        .map(|s| s.id())
        .collect::<HashSet<SpeciesId>>();

    for phenotype in pop.iter() {
        assert_ne!(
            phenotype.species(),
            empty_id,
            "{label}: expected all phenotypes to be assigned to a species, but found one with None"
        );
        assert!(
            species.contains(&phenotype.species()),
            "{label}: expected all phenotypes to be assigned to a valid species, but found one with invalid species ID {:?}",
            phenotype.species()
        );
    }
}

pub fn assert_within_budget<C: Chromosome, T: Clone>(
    result: &Generation<C, T>,
    budget: usize,
    label: &str,
) {
    assert!(
        result.index() < budget,
        "{label}: convergence budget exceeded ({} / {budget})",
        result.index()
    );
}

/// Assert that every phenotype in the population has a non-`None`
/// genotype and a finite score.
pub fn assert_population_integrity<C: Chromosome, T: Clone>(
    result: &Generation<C, T>,
    expected_size: usize,
) {
    let pop = result.population();
    assert_eq!(
        pop.len(),
        expected_size,
        "population size drifted: got {}, expected {expected_size}",
        pop.len()
    );

    for (i, phenotype) in pop.iter().enumerate() {
        // .genotype() panics on None — this implicitly asserts.
        let _ = phenotype.genotype();
        if let Some(score) = phenotype.score() {
            assert!(
                score.iter().all(|s| s.is_finite()),
                "phenotype {i} has non-finite score: {:?}",
                score
            );
        }
    }
}

/// Assert convergence within a budget for a maximize-to-target problem.
/// Equivalent to a tightened `engine.run()` — fails loudly if either
/// the optimum isn't reached or the budget is blown.
pub fn assert_converged_to_target<C, T, F>(
    result: &Generation<C, T>,
    budget: usize,
    label: &str,
    score_check: F,
) where
    C: Chromosome,
    T: Clone,
    F: Fn(&Score) -> bool,
{
    assert!(
        score_check(result.score()),
        "{label}: did not reach target score (got {:?} at gen {})",
        result.score(),
        result.index()
    );
    assert_within_budget(result, budget, label);
}

/// Assert that two trajectories (per-gen score sequences) are
/// element-wise equal. Used by determinism tests to catch RNG drift.
pub fn assert_identical_trajectory(left: &[f32], right: &[f32], label: &str) {
    assert_eq!(
        left.len(),
        right.len(),
        "{label}: trajectory lengths differ ({} vs {})",
        left.len(),
        right.len()
    );
    for (i, (a, b)) in left.iter().zip(right.iter()).enumerate() {
        assert_eq!(
            a, b,
            "{label}: trajectory diverges at gen {i}: {a} vs {b}\n  left:  {:?}\n  right: {:?}",
            left, right
        );
    }
}
