//! Alterer harness and property assertions.
//!
//! [`run_alterer`] drives a single [`Alterer`] over a cloned population
//! through the public `Alterer::alter` path (rate expressions, metrics
//! and all), so the assertions below exercise exactly what the engine
//! runs. Each assertion compares a `before` population against the
//! `after` one and holds for *any* alterer that claims the property.

#![allow(dead_code)]

use std::cmp::Ordering;
use std::fmt::Debug;

use radiate_core::{
    Alterer, BoundedGene, Chromosome, Codec, Gene, MetricSet, Population, random_provider,
};

use crate::MockEcosystem;

/// A seeded, unscored population of `size` individuals from `codec`.
/// Same `seed` → same population, so tests can share a fixed baseline.
pub fn alter_population<C, T>(
    codec: impl Codec<C, T> + 'static,
    size: usize,
    seed: u64,
) -> Population<C>
where
    C: Chromosome + Clone,
{
    random_provider::scoped_seed(seed, || {
        MockEcosystem::builder(codec)
            .pop_size(size)
            .build_population()
    })
}

/// Apply `alterer` once to a clone of `population` under `seed` and
/// return the altered clone. `population` itself is left untouched so it
/// can serve as the `before` side of a property assertion.
pub fn run_alterer<C>(mut alterer: Alterer<C>, population: &Population<C>, seed: u64) -> Population<C>
where
    C: Chromosome + Clone,
{
    let mut altered = population.clone();
    let mut metrics = MetricSet::new();

    random_provider::scoped_seed(seed, || {
        alterer
            .alter(altered.as_mut(), &mut metrics, 0)
            .unwrap_or_else(|e| panic!("{}: alter failed: {e}", alterer.name()));
    });

    altered
}

/// Every gene allele lies within its gene's (inclusive) bounds. NaN
/// alleles fail this check too, since they compare false to both ends.
pub fn assert_genes_in_bounds<C>(population: &Population<C>, label: &str)
where
    C: Chromosome,
    C::Gene: BoundedGene,
    <C::Gene as Gene>::Allele: PartialOrd + Debug,
{
    for (p, phenotype) in population.iter().enumerate() {
        for (c, chromosome) in phenotype.genotype().iter().enumerate() {
            for (g, gene) in chromosome.iter().enumerate() {
                let (min, max) = gene.bound_range();
                let allele = gene.allele();
                assert!(
                    allele >= min && allele <= max,
                    "{label}: gene [{p}][{c}][{g}] = {allele:?} is outside bounds [{min:?}, {max:?}]"
                );
            }
        }
    }
}

/// Population size, chromosomes per genotype, and genes per chromosome
/// are all unchanged.
pub fn assert_shape_preserved<C: Chromosome>(
    before: &Population<C>,
    after: &Population<C>,
    label: &str,
) {
    assert_eq!(
        before.len(),
        after.len(),
        "{label}: population size changed"
    );

    for (p, (b, a)) in before.iter().zip(after.iter()).enumerate() {
        let (b, a) = (b.genotype(), a.genotype());
        assert_eq!(
            b.len(),
            a.len(),
            "{label}: phenotype {p} chromosome count changed"
        );

        for (c, (bc, ac)) in b.iter().zip(a.iter()).enumerate() {
            assert_eq!(
                bc.len(),
                ac.len(),
                "{label}: phenotype {p} chromosome {c} length changed"
            );
        }
    }
}

/// Each chromosome holds the same multiset of alleles before and after —
/// genes may move within a chromosome but never appear, vanish, or
/// change value. The contract for reordering mutators (swap, scramble,
/// inversion) and permutation crossovers (PMX, edge recombination).
pub fn assert_alleles_permuted<C>(before: &Population<C>, after: &Population<C>, label: &str)
where
    C: Chromosome,
    <C::Gene as Gene>::Allele: PartialOrd + Clone + Debug,
{
    assert_shape_preserved(before, after, label);

    for (p, (b, a)) in before.iter().zip(after.iter()).enumerate() {
        for (c, (bc, ac)) in b.genotype().iter().zip(a.genotype().iter()).enumerate() {
            let (expected, actual) = (sorted_alleles(bc.iter()), sorted_alleles(ac.iter()));
            assert_eq!(
                expected, actual,
                "{label}: phenotype {p} chromosome {c} allele multiset changed"
            );
        }
    }
}

/// For every (chromosome, gene) position, the multiset of alleles across
/// the whole population is unchanged — genes only trade places between
/// individuals at the same locus. The contract for positional-swap
/// crossovers (uniform, shuffle, multi-point).
pub fn assert_loci_preserved<C>(before: &Population<C>, after: &Population<C>, label: &str)
where
    C: Chromosome,
    <C::Gene as Gene>::Allele: PartialOrd + Clone + Debug,
{
    assert_shape_preserved(before, after, label);

    let Some(first) = before.iter().next() else {
        return;
    };

    for (c, chromosome) in first.genotype().iter().enumerate() {
        for g in 0..chromosome.len() {
            let locus = |pop: &Population<C>| {
                sorted_alleles(pop.iter().filter_map(|p| p.genotype()[c].get(g)))
            };
            assert_eq!(
                locus(before),
                locus(after),
                "{label}: allele multiset at locus [{c}][{g}] changed"
            );
        }
    }
}

/// Genotypes are identical before and after.
pub fn assert_genotypes_unchanged<C>(before: &Population<C>, after: &Population<C>, label: &str)
where
    C: Chromosome + PartialEq + Debug,
{
    assert_eq!(
        before.len(),
        after.len(),
        "{label}: population size changed"
    );

    for (p, (b, a)) in before.iter().zip(after.iter()).enumerate() {
        assert_eq!(
            b.genotype(),
            a.genotype(),
            "{label}: phenotype {p} genotype changed"
        );
    }
}

/// At least one genotype differs between before and after. Guards the
/// other properties against passing vacuously on an alterer that
/// silently does nothing.
pub fn assert_genotypes_changed<C>(before: &Population<C>, after: &Population<C>, label: &str)
where
    C: Chromosome + PartialEq,
{
    let changed = before
        .iter()
        .zip(after.iter())
        .any(|(b, a)| b.genotype() != a.genotype());

    assert!(changed, "{label}: expected the alterer to change at least one genotype");
}

/// Two fresh alterers from `make`, run on the same population under the
/// same seed, produce identical genotypes.
///
/// Takes a factory rather than an [`Alterer`] because a cloned mutator
/// shares its `Arc` with the original, and `Alterer::alter` needs unique
/// ownership of it.
pub fn assert_alterer_deterministic<C>(
    make: impl Fn() -> Alterer<C>,
    population: &Population<C>,
    seed: u64,
    label: &str,
) where
    C: Chromosome + Clone + PartialEq + Debug,
{
    let first = run_alterer(make(), population, seed);
    let second = run_alterer(make(), population, seed);
    assert_genotypes_unchanged(&first, &second, label);
}

fn sorted_alleles<'a, G, I>(genes: I) -> Vec<G::Allele>
where
    G: Gene + 'a,
    G::Allele: PartialOrd + Clone,
    I: Iterator<Item = &'a G>,
{
    let mut alleles = genes.map(|g| g.allele().clone()).collect::<Vec<_>>();
    alleles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    alleles
}
