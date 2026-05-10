#[cfg(test)]
mod recombine_step_tests {
    use radiate_test::*;
    use radiate_core::*;
    use radiate_engines::*;

    #[test]
    fn non_species_preserves_pop_size() {
        seeded(1, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
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
    fn non_species_no_dropped_phenotypes() {
        seeded(2, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(40)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_recombine_step(10, 30, minimize(), default_float_alters());

            step.execute(0, &mut eco, &mut metrics).unwrap();

            for (i, p) in eco.population().iter().enumerate() {
                let _ = p.genotype();
                assert_eq!(p.genotype().len(), 1, "phenotype {i} lost its chromosome");
            }
        });
    }

    #[test]
    fn species_preserves_pop_size() {
        seeded(3, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
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

    #[test]
    fn species_no_dropped_phenotypes() {
        seeded(4, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(30)
                .scores_linear()
                .with_species(&[15, 10, 5])
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_recombine_step(10, 20, minimize(), default_float_alters());

            step.execute(0, &mut eco, &mut metrics).unwrap();

            for (i, p) in eco.population().iter().enumerate() {
                let _ = p.genotype();
                assert_eq!(p.genotype().len(), 1, "phenotype {i} lost its chromosome");
            }
        });
    }

    /// Species path with a 1-member species. Edge case where per-species
    /// sort and selection have nothing to do; the unified walk still has
    /// to produce a valid pop.
    #[test]
    fn species_with_singleton_species_no_panic() {
        seeded(5, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(2, -1.0..1.0))
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

    /// Both objectives produce valid populations. Catches sign-error
    /// regressions in score-sorted-then-bumped logic.
    #[test]
    fn both_objectives_produce_valid_populations() {
        for (seed, obj) in [(10u64, minimize()), (11, maximize())] {
            seeded(seed, || {
                let mut eco = MockEcosystem::new(FloatCodec::vector(2, -1.0..1.0))
                    .pop_size(20)
                    .scores_linear()
                    .build();
                let mut metrics = MetricSet::new();
                let mut step = mock_recombine_step(5, 15, obj.clone(), default_float_alters());

                step.execute(0, &mut eco, &mut metrics).unwrap();

                assert_eq!(eco.population().len(), 20);
                for p in eco.population().iter() {
                    let _ = p.genotype();
                }
            });
        }
    }

    /// Repeated calls on the same `RecombineStep` produce a stable pop
    /// size. Catches `VersionedCounts` not resetting between calls and
    /// per-call state leaks.
    #[test]
    fn repeated_calls_stable_pop_size() {
        seeded(20, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
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
}
