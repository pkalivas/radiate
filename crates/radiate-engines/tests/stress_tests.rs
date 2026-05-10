//! Stress tests: large populations, long runs, many species. These
//! verify the engine stays stable and reasonably fast under load — they
//! aren't asserting *exact* convergence, just that nothing panics, no
//! NaN/None state leaks in, and pop-size invariants hold.
//!
//! These tests are slower than the convergence tests (multi-second).

#[path = "common/mod.rs"]
mod common;

#[cfg(test)]
mod stress_tests {
    use super::common::*;
    use radiate_core::*;
    use radiate_engines::*;

    #[test]
    fn long_run_large_pop_stable() {
        const POP_SIZE: usize = 500;
        const GENS: usize = 1000;

        seeded(31337, || {
            let problem = Sphere::new(8, 10.0);
            let engine = GeneticEngine::builder()
                .minimizing()
                .population_size(POP_SIZE)
                .codec(problem.codec())
                .fitness_fn(problem.fitness_fn())
                .alter(alters![
                    BlendCrossover::new(0.5, 0.5),
                    GaussianMutator::new(0.05)
                ])
                .build();

            let result = engine.iter().limit(Limit::Generation(GENS)).last().unwrap();
            let best = result.score().as_f32();

            assert_population_integrity(&result, POP_SIZE);
            assert!(best.is_finite(), "best score is non-finite: {best}");
            assert!(
                best < 100.0,
                "best score {best} suggests engine made no progress after {GENS} gens"
            );
        });
    }

    #[test]
    fn many_species_stable() {
        const POP_SIZE: usize = 200;
        const GENS: usize = 300;
        const SPECIES_COUNT: usize = 70;

        seeded(424242, || {
            let engine = speciated_sphere_engine(5, POP_SIZE, 0.1);
            let result = engine.iter().limit(Limit::Generation(GENS)).last().unwrap();

            assert_has_species(&result, "many species stable");
            assert_has_n_species(&result, SPECIES_COUNT, "many species stable");
            assert_population_integrity(&result, POP_SIZE);
        });
    }

    #[test]
    fn same_seed_produces_identical_trajectory() {
        const SEED: u64 = 99999;
        const GENS: usize = 50;

        let scores_a = run_and_collect_scores(SEED, GENS);
        let scores_b = run_and_collect_scores(SEED, GENS);

        assert_identical_trajectory(&scores_a, &scores_b, "same-seed run pair");
    }

    fn run_and_collect_scores(seed: u64, gens: usize) -> Vec<f32> {
        seeded(seed, || {
            sphere_engine(4)
                .iter()
                .limit(Limit::Generation(gens))
                .map(|ctx| ctx.score().as_f32())
                .collect()
        })
    }
}
