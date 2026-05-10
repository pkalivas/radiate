//! Species-path coverage. The `RecombineStep::create_with_species` branch
//! is structurally distinct from the non-species path

#[cfg(test)]
mod species_tests {
    use radiate_core::*;
    use radiate_engines::*;
    use radiate_test::*;
    use rstest::*;

    #[rstest]
    #[case(101, 0.05, 300)]
    #[case(202, 0.05, 300)]
    #[case(303, 0.05, 300)]
    fn speciated_regression_converges(
        #[case] seed: u64,
        #[case] threshold: f32,
        #[case] budget: usize,
    ) {
        seeded(seed, || {
            // Custom inline build because this test exercises
            // selector & alter combinations the helpers don't expose.
            let test_inputs = (-10..=10).map(|i| i as f32 / 5.0).collect::<Vec<f32>>();
            let targets = test_inputs
                .iter()
                .map(|x| 2.0 * x + 1.0)
                .collect::<Vec<f32>>();

            let engine = GeneticEngine::builder()
                .minimizing()
                .population_size(80)
                .codec(FloatCodec::vector(2, -10.0..10.0))
                .diversity(EuclideanDistance)
                .species_threshold(0.5)
                .survivor_selector(TournamentSelector::new(3))
                .offspring_selector(BoltzmannSelector::new(4.0))
                .alter(alters![
                    BlendCrossover::new(0.6, 0.5),
                    GaussianMutator::new(0.05)
                ])
                .fitness_fn(move |coeffs: Vec<f32>| -> f32 {
                    test_inputs
                        .iter()
                        .zip(targets.iter())
                        .map(|(x, y)| {
                            let pred = coeffs[0] * x + coeffs[1];
                            (pred - y).powi(2)
                        })
                        .sum::<f32>()
                        / test_inputs.len() as f32
                })
                .build();

            let result = engine
                .iter()
                .limit(vec![
                    Limit::Generation(budget),
                    Limit::Score(threshold.into()),
                ])
                .last()
                .unwrap();

            assert!(
                result.score().as_f32() < threshold,
                "speciated regression MSE {} did not reach {} within {} gens",
                result.score().as_f32(),
                threshold,
                budget,
            );
            assert_within_budget(&result, budget, "speciated regression");
            assert_has_species(result.ecosystem(), "speciated regression");
        });
    }

    /// Pop-integrity invariants every generation when species are active.
    /// Runs a short engine, then audits the final population state.
    #[test]
    fn speciated_population_integrity() {
        const POP_SIZE: usize = 100;
        const GENS: usize = 50;

        seeded(7777, || {
            let engine = speciated_sphere_engine(3, POP_SIZE, 0.5);
            let result = engine.iter().limit(GENS).last().unwrap();

            assert_population_integrity(&result, POP_SIZE);
        });
    }

    /// Edge case: very small population. Speciation can fail in interesting
    /// ways with K phenotypes < typical species count.
    #[test]
    fn speciated_small_population_no_panic() {
        const POP_SIZE: usize = 3;
        const GENS: usize = 100;

        seeded(1010, || {
            let engine = speciated_sphere_engine(2, POP_SIZE, 0.3);
            let result = engine.iter().limit(GENS).last().unwrap();

            assert_eq!(result.population().len(), POP_SIZE);
        });
    }
}
