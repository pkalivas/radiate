//! Property tests run across every alterer in the crate.
//!
//! Each `rstest` matrix takes an alterer *factory* `fn(rate) -> Alterer`,
//! so one case list covers several properties (bounds, shape, rate 0,
//! determinism) and the rate-0 test reuses the same constructors.

use radiate_alters::*;
use radiate_core::*;
use radiate_test::*;
use rstest::rstest;

const POP_SIZE: usize = 20;
const POP_SEED: u64 = 7;
const ALTER_SEED: u64 = 42;

type Make<C> = fn(f32) -> Alterer<C>;

fn float_population() -> Population<FloatChromosome<f32>> {
    alter_population(FloatCodec::vector(8, -1.0..1.0), POP_SIZE, POP_SEED)
}

fn int_population() -> Population<IntChromosome<i32>> {
    alter_population(IntCodec::vector(8, 0..100), POP_SIZE, POP_SEED)
}

fn bit_population() -> Population<BitChromosome> {
    alter_population(BitCodec::vector(16), POP_SIZE, POP_SEED)
}

fn permutation_population() -> Population<PermutationChromosome<usize>> {
    alter_population(PermutationCodec::new((0..10).collect()), POP_SIZE, POP_SEED)
}

/// Shared checks for every alterer: shape is preserved at a working rate,
/// the alterer actually does something at rate 1, rate 0 is a no-op, and
/// a fixed seed reproduces the same output.
fn assert_common_properties<C>(make: Make<C>, population: Population<C>)
where
    C: Chromosome + Clone + PartialEq + std::fmt::Debug,
{
    let label = make(0.0).name().to_string();

    let altered = run_alterer(make(0.5), &population, ALTER_SEED);
    assert_shape_preserved(&population, &altered, &label);

    let altered = run_alterer(make(1.0), &population, ALTER_SEED);
    assert_genotypes_changed(&population, &altered, &label);

    let altered = run_alterer(make(0.0), &population, ALTER_SEED);
    assert_genotypes_unchanged(&population, &altered, &format!("{label} @ rate 0"));

    assert_alterer_deterministic(|| make(0.5), &population, ALTER_SEED, &label);
}

#[rstest]
#[case::blend(|r| BlendCrossover::new(r, 0.5).into_alterer())]
#[case::intermediate(|r| IntermediateCrossover::new(r, 0.5).into_alterer())]
#[case::mean(|r| MeanCrossover::new(r).into_alterer())]
#[case::simulated_binary(|r| SimulatedBinaryCrossover::new(r, 1.0).into_alterer())]
#[case::uniform_crossover(|r| UniformCrossover::new(r).into_alterer())]
#[case::multi_point(|r| MultiPointCrossover::new(r, 2).into_alterer())]
#[case::shuffle(|r| ShuffleCrossover::new(r).into_alterer())]
#[case::gaussian(|r| GaussianMutator::new(r).into_alterer())]
#[case::jitter(|r| JitterMutator::new(r, 0.5).into_alterer())]
#[case::polynomial(|r| PolynomialMutator::new(r, 20.0).into_alterer())]
#[case::uniform_mutator(|r| UniformMutator::new(r).into_alterer())]
#[case::arithmetic(|r| ArithmeticMutator::new(r).into_alterer())]
#[case::swap(|r| SwapMutator::new(r).into_alterer())]
#[case::scramble(|r| ScrambleMutator::new(r).into_alterer())]
#[case::inversion(|r| InversionMutator::new(r).into_alterer())]
fn float_alterer_properties(#[case] make: Make<FloatChromosome<f32>>) {
    let population = float_population();
    let label = make(0.0).name().to_string();

    for rate in [0.5, 1.0] {
        let altered = run_alterer(make(rate), &population, ALTER_SEED);
        assert_genes_in_bounds(&altered, &format!("{label} @ rate {rate}"));
    }

    assert_common_properties(make, population);
}

#[rstest]
#[case::mean(|r| MeanCrossover::new(r).into_alterer())]
#[case::uniform_crossover(|r| UniformCrossover::new(r).into_alterer())]
#[case::multi_point(|r| MultiPointCrossover::new(r, 2).into_alterer())]
#[case::shuffle(|r| ShuffleCrossover::new(r).into_alterer())]
#[case::uniform_mutator(|r| UniformMutator::new(r).into_alterer())]
#[case::arithmetic(|r| ArithmeticMutator::new(r).into_alterer())]
#[case::swap(|r| SwapMutator::new(r).into_alterer())]
#[case::scramble(|r| ScrambleMutator::new(r).into_alterer())]
#[case::inversion(|r| InversionMutator::new(r).into_alterer())]
fn int_alterer_properties(#[case] make: Make<IntChromosome<i32>>) {
    let population = int_population();
    let label = make(0.0).name().to_string();

    for rate in [0.5, 1.0] {
        let altered = run_alterer(make(rate), &population, ALTER_SEED);
        assert_genes_in_bounds(&altered, &format!("{label} @ rate {rate}"));
    }

    assert_common_properties(make, population);
}

#[rstest]
#[case::bit_flip(|r| BitFlipMutator::new(r).into_alterer())]
#[case::uniform_mutator(|r| UniformMutator::new(r).into_alterer())]
#[case::uniform_crossover(|r| UniformCrossover::new(r).into_alterer())]
#[case::multi_point(|r| MultiPointCrossover::new(r, 2).into_alterer())]
#[case::shuffle(|r| ShuffleCrossover::new(r).into_alterer())]
#[case::swap(|r| SwapMutator::new(r).into_alterer())]
#[case::scramble(|r| ScrambleMutator::new(r).into_alterer())]
#[case::inversion(|r| InversionMutator::new(r).into_alterer())]
fn bit_alterer_properties(#[case] make: Make<BitChromosome>) {
    assert_common_properties(make, bit_population());
}

#[rstest]
#[case::swap(|r| SwapMutator::new(r).into_alterer())]
#[case::scramble(|r| ScrambleMutator::new(r).into_alterer())]
#[case::inversion(|r| InversionMutator::new(r).into_alterer())]
#[case::pmx(|r| PMXCrossover::new(r).into_alterer())]
#[case::order(|r| OrderCrossover::new(r).into_alterer())]
#[case::edge_recombination(|r| EdgeRecombinationCrossover::new(r).into_alterer())]
fn permutation_alterers_preserve_alleles(#[case] make: Make<PermutationChromosome<usize>>) {
    let population = permutation_population();
    let label = make(0.0).name().to_string();

    for rate in [0.5, 1.0] {
        let altered = run_alterer(make(rate), &population, ALTER_SEED);
        assert_alleles_permuted(&population, &altered, &format!("{label} @ rate {rate}"));
    }

    assert_common_properties(make, population);
}

/// Reordering mutators must not change allele values on non-permutation
/// chromosomes either.
#[rstest]
#[case::swap(|r| SwapMutator::new(r).into_alterer())]
#[case::scramble(|r| ScrambleMutator::new(r).into_alterer())]
#[case::inversion(|r| InversionMutator::new(r).into_alterer())]
fn reordering_mutators_preserve_int_alleles(#[case] make: Make<IntChromosome<i32>>) {
    let population = int_population();
    let altered = run_alterer(make(1.0), &population, ALTER_SEED);
    assert_alleles_permuted(&population, &altered, make(0.0).name());
}

/// Positional-swap crossovers only trade genes between individuals at the
/// same locus, so each locus keeps its allele multiset across the population.
#[rstest]
#[case::uniform_crossover(|r| UniformCrossover::new(r).into_alterer())]
#[case::multi_point(|r| MultiPointCrossover::new(r, 2).into_alterer())]
#[case::shuffle(|r| ShuffleCrossover::new(r).into_alterer())]
fn positional_crossovers_preserve_loci(#[case] make: Make<IntChromosome<i32>>) {
    let population = int_population();

    for rate in [0.5, 1.0] {
        let altered = run_alterer(make(rate), &population, ALTER_SEED);
        assert_loci_preserved(&population, &altered, make(0.0).name());
    }
}
