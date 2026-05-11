#[cfg(test)]
mod recombine_step_tests {
    use radiate_core::*;
    use radiate_engines::*;
    use radiate_test::*;

    #[test]
    fn recombine_non_species_preserves_pop_size() {
        seeded(1, || {
            let mut eco = MockEcosystem::builder(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(50)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_recombine_step(15, 35, minimize(), default_float_alters());

            step.execute(0, &mut eco, &mut metrics).unwrap();

            assert_eq!(eco.population().len(), 50);
        });
    }

    #[test]
    fn recombine_species_preserves_pop_size() {
        seeded(3, || {
            let mut eco = MockEcosystem::builder(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(30)
                .scores_linear()
                .with_species(&[15, 10, 5])
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_recombine_step(10, 20, minimize(), default_float_alters());

            step.execute(0, &mut eco, &mut metrics).unwrap();

            assert_eq!(eco.population().len(), 30);
        });
    }

    /// Species path with a 1-member species. Edge case where per-species
    /// sort and selection have nothing to do; the unified walk still has
    /// to produce a valid pop.
    #[test]
    fn recombine_species_with_singleton_species_no_panic() {
        seeded(5, || {
            let mut eco = MockEcosystem::builder(FloatCodec::vector(2, -1.0..1.0))
                .pop_size(10)
                .scores_linear()
                .with_species(&[1, 4, 5])
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_recombine_step(3, 7, minimize(), default_float_alters());

            step.execute(0, &mut eco, &mut metrics).unwrap();

            assert_eq!(eco.population().len(), 10);
        });
    }

    /// Both objectives produce valid populations.
    #[test]
    fn recombine_both_objectives_produce_valid_populations() {
        for (seed, obj) in [(10u64, minimize()), (11, maximize())] {
            seeded(seed, || {
                let mut eco = MockEcosystem::builder(FloatCodec::vector(2, -1.0..1.0))
                    .pop_size(20)
                    .scores_linear()
                    .build();
                let mut metrics = MetricSet::new();
                let mut step = mock_recombine_step(5, 15, obj.clone(), default_float_alters());

                step.execute(0, &mut eco, &mut metrics).unwrap();

                assert_eq!(eco.population().len(), 20);
            });
        }
    }

    /// Repeated calls on the same `RecombineStep` produce a stable pop
    /// size. Catches `VersionedCounts` not resetting between calls and
    /// per-call state leaks.
    #[test]
    fn recombine_repeated_calls_stable_pop_size() {
        seeded(20, || {
            let mut eco = MockEcosystem::builder(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(40)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_recombine_step(10, 30, minimize(), default_float_alters());

            for generation in 0..5 {
                // Re-score before each generation since alters wipe scores.
                for (i, p) in eco.population_mut().iter_mut().enumerate() {
                    if p.score().is_none() {
                        p.set_score(Some(Score::from(i as f32)));
                    }
                }

                step.execute(generation, &mut eco, &mut metrics).unwrap();
                assert_eq!(eco.population().len(), 40, "drifted at gen {generation}");
            }
        });
    }

    // --- Species Step tests ---

    /// Fresh population (no species) should bootstrap into at least one
    /// species after the first speciate run.
    #[test]
    fn speciate_fresh_population_creates_initial_species() {
        seeded(100, || {
            let mut eco = MockEcosystem::builder(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(20)
                .scores_linear()
                .build();

            let mut metrics = MetricSet::new();
            let mut step = mock_speciate_step(0.5, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            assert_has_species(&eco, "fresh pop should produce at least one species");
        });
    }

    /// Loose threshold: distance threshold so high that all phenotypes
    /// fall into the same species. Distinct from fresh-population
    /// because it asserts the *count* — exactly one species.
    #[test]
    fn speciate_loose_threshold_produces_single_species() {
        seeded(101, || {
            let mut eco = MockEcosystem::builder(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(30)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            // Threshold > maximum possible Euclidean distance for 3-gene floats in [-1, 1].
            let mut step = mock_speciate_step(1_000.0, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            assert_species_count(
                &eco,
                1,
                "loose threshold should produce exactly one species",
            );
            assert_population_speciated(&eco, "loose threshold should produce exactly one species");
        });
    }

    #[test]
    fn speciate_produces_adjusted_fitness_values() {
        seeded(106, || {
            const POP_SIZE: usize = 300;
            let mut eco = MockEcosystem::builder(FloatCodec::vector(3, -10.0..10.0))
                .pop_size(POP_SIZE)
                .scores_random(-5.0..5.0)
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_speciate_step(4.0, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            let species = eco.species().expect("species created");

            let mut species_score_sum = 0.0;
            for spec in species.iter() {
                let adjusted_score = spec.adj_score().expect("adjusted score set");
                species_score_sum += adjusted_score.as_f32();
            }

            assert_species_count(
                &eco,
                24,
                "adjusted fitness values should be produced even with a tight threshold",
            );
            assert!(
                (species_score_sum - 1.0).abs() < 1e-6,
                "adjusted fitness values should be scaled down (got sum {species_score_sum})"
            );
        });
    }
}
