//! Convergence tests with strict generation budgets.

#[path = "common/mod.rs"]
mod common;

#[cfg(test)]
mod convergence_tests {
    use super::common::*;
    use radiate_core::*;
    use radiate_engines::*;
    use rstest::*;

    /// One-Max: maximize sum of bits in a fixed-length bitstring.
    /// Trivially solvable; used here as a smoke test for selector × alterer
    /// matrix combinations. Not factored through `onemax_engine()` because
    /// each case overrides the survivor + offspring selectors.
    #[rstest]
    #[case::tournament_uniform(
        Box::new(TournamentSelector::new(3)) as Box<dyn Select<BitChromosome>>,
        Box::new(RouletteSelector::new()) as Box<dyn Select<BitChromosome>>,
        100, 80,
    )]
    #[case::tournament_boltzmann(
        Box::new(TournamentSelector::new(3)) as Box<dyn Select<BitChromosome>>,
        Box::new(BoltzmannSelector::new(4.0)) as Box<dyn Select<BitChromosome>>,
        100, 80,
    )]
    #[case::elite_roulette(
        Box::new(EliteSelector::new()) as Box<dyn Select<BitChromosome>>,
        Box::new(RouletteSelector::new()) as Box<dyn Select<BitChromosome>>,
        100, 80,
    )]
    fn onemax_converges_within_budget(
        #[case] survivor: Box<dyn Select<BitChromosome>>,
        #[case] offspring: Box<dyn Select<BitChromosome>>,
        #[case] seed: u64,
        #[case] budget: usize,
    ) {
        const N_BITS: usize = 30;
        let problem = OneMax::new(N_BITS);

        seeded(seed, || {
            let engine = GeneticEngine::builder()
                .codec(problem.codec())
                .fitness_fn(problem.fitness_fn())
                .boxed_survivor_selector(survivor)
                .boxed_offspring_selector(offspring)
                .alter(alters![
                    UniformCrossover::new(0.7),
                    UniformMutator::new(0.05)
                ])
                .build();

            let result = engine
                .iter()
                .limit(Limit::Generation(budget))
                .until_score(problem.optimum())
                .last()
                .unwrap();

            let count = result.value().iter().filter(|b| **b).count();
            assert_eq!(
                count,
                N_BITS,
                "OneMax did not reach optimum within budget (got {count}/{N_BITS} at gen {})",
                result.index()
            );
            assert_within_budget(&result, budget, "OneMax");
        });
    }

    /// IntCodec minimize-to-zero. Same problem `engine_can_minimize` uses but
    /// with explicit budget + seed, so a refactor that triples convergence
    /// time fails this test.
    #[rstest]
    #[case(7, 200)]
    #[case(99, 200)]
    #[case(1234, 200)]
    fn int_minimize_converges_within_budget(#[case] seed: u64, #[case] budget: usize) {
        seeded(seed, || {
            let engine = int_minimize_engine(5, 100);

            let result = engine
                .iter()
                .limit(Limit::Generation(budget))
                .until_score(0)
                .last()
                .unwrap();

            assert_eq!(result.value().iter().sum::<i32>(), 0);
            assert_within_budget(&result, budget, "int-minimize-to-zero");
        });
    }

    /// Float regression: classic minimize-MSE on y = x² over [-2, 2]. Uses
    /// real-valued chromosomes + Gaussian mutation. Catches regressions that
    /// affect float-pathway selectors and alterers.
    #[rstest]
    #[case(11, 0.01, 200)]
    #[case(22, 0.01, 200)]
    fn float_regression_converges(
        #[case] seed: u64,
        #[case] threshold: f32,
        #[case] budget: usize,
    ) {
        seeded(seed, || {
            let engine = quadratic_regression_engine(41);

            let result = engine
                .iter()
                .limit(Limit::Generation(budget))
                .until_score(threshold)
                .last()
                .unwrap();

            assert!(
                result.score().as_f32() < threshold,
                "regression MSE {} did not reach {} within {} gens",
                result.score().as_f32(),
                threshold,
                budget,
            );
        });
    }
}
