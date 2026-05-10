#[cfg(test)]
mod speciate_step_tests {
    use radiate_test::*;
    use radiate_core::*;
    use radiate_engines::*;

    /// Fresh population (no species) should bootstrap into at least one
    /// species after the first speciate run.
    #[test]
    fn fresh_population_creates_initial_species() {
        seeded(100, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(20)
                .scores_linear()
                .build();
            assert!(
                eco.species().is_none() || eco.species().unwrap().is_empty(),
                "precondition: ecosystem starts with no species"
            );

            let mut metrics = MetricSet::new();
            let mut step = mock_speciate_step(0.5, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            assert!(
                eco.species().is_some_and(|s| !s.is_empty()),
                "fresh pop should produce at least one species"
            );
        });
    }

    /// Loose threshold: distance threshold so high that all phenotypes
    /// fall into the same species. Distinct from fresh-population
    /// because it asserts the *count* — exactly one species.
    #[test]
    fn loose_threshold_produces_single_species() {
        seeded(101, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(30)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            // Threshold > maximum possible Euclidean distance for 3-gene floats in [-1, 1].
            let mut step = mock_speciate_step(1_000.0, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            let species = eco.species().expect("species created");
            assert_eq!(
                species.len(),
                1,
                "loose threshold should collapse pop into 1 species, got {}",
                species.len()
            );
        });
    }

    /// Tight threshold: distance threshold so small that effectively
    /// every phenotype gets its own species. Catches regressions where
    /// new-species creation is suppressed when distance is barely > 0.
    #[test]
    fn tight_threshold_produces_many_species() {
        seeded(102, || {
            const POP_SIZE: usize = 20;
            let mut eco = MockEcosystem::new(FloatCodec::vector(5, -10.0..10.0))
                .pop_size(POP_SIZE)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_speciate_step(1e-9, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            let species = eco.species().expect("species created");
            // Each random float genotype is distinct → effectively one
            // species per phenotype with a tight threshold.
            assert!(
                species.len() >= POP_SIZE / 2,
                "tight threshold should produce many species, got {}/{POP_SIZE}",
                species.len()
            );
        });
    }

    /// Pop size invariant: speciate doesn't drop or duplicate phenotypes.
    /// Catches regressions where the "swap empty pop in/out" trick
    /// during chunked processing leaks or duplicates.
    #[test]
    fn pop_size_invariant_through_speciate() {
        seeded(103, || {
            const POP_SIZE: usize = 50;
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(POP_SIZE)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_speciate_step(0.5, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            assert_eq!(eco.population().len(), POP_SIZE);
        });
    }

    /// Sum-of-members invariant: every phenotype is a member of exactly
    /// one species. The species member-count totals must equal pop size.
    /// Catches regressions where assignments miss phenotypes (orphans)
    /// or double-assign them.
    #[test]
    fn sum_of_species_members_equals_pop_size() {
        seeded(104, || {
            const POP_SIZE: usize = 40;
            let mut eco = MockEcosystem::new(FloatCodec::vector(4, -5.0..5.0))
                .pop_size(POP_SIZE)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_speciate_step(2.0, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();

            let species = eco.species().expect("species created");
            let total_members: usize = species.iter().map(|s| s.len()).sum();
            assert_eq!(
                total_members, POP_SIZE,
                "every phenotype should belong to exactly one species \
                 (got {total_members} members for {POP_SIZE} phenotypes)"
            );
        });
    }

    /// Repeated speciate calls don't grow species count unboundedly when
    /// the population doesn't change.
    #[test]
    fn repeated_speciate_stable_species_count() {
        seeded(105, || {
            let mut eco = MockEcosystem::new(FloatCodec::vector(3, -1.0..1.0))
                .pop_size(30)
                .scores_linear()
                .build();
            let mut metrics = MetricSet::new();
            let mut step = mock_speciate_step(0.5, EuclideanDistance);

            step.execute(0, &mut eco, &mut metrics).unwrap();
            let species_after_first = eco.species().unwrap().len();

            // Same pop, run speciate 3 more times. Species count should
            // not blow up.
            for generation in 1..=3 {
                step.execute(generation, &mut eco, &mut metrics).unwrap();
            }
            let species_after_repeated = eco.species().unwrap().len();

            assert!(
                species_after_repeated <= species_after_first * 2,
                "species count grew unboundedly under repeat: {species_after_first} → {species_after_repeated}"
            );
        });
    }
}
