#[cfg(test)]
mod engine_tests {
    use radiate_core::*;
    use radiate_engines::*;
    use std::time::Duration;

    #[test]
    fn engine_can_minimize() {
        const EXPECTED_SUM: i32 = 0;
        const EXPECTED_GENS: usize = 60;

        random_provider::scoped_seed(42, || {
            let engine = GeneticEngine::builder()
                .minimizing()
                .codec(IntCodec::vector(5, 0..100))
                .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
                .build();

            let result = engine.iter().until_score(0).last().unwrap();
            let best = result.value();

            assert_eq!(best.iter().sum::<i32>(), EXPECTED_SUM);
            assert_eq!(result.index(), EXPECTED_GENS);
        });
    }

    #[test]
    fn engine_can_maximize() {
        const EXPECTED_SUM: i32 = 500;
        const BUDGET: usize = 500;

        random_provider::scoped_seed(43, || {
            let mut engine = GeneticEngine::builder()
                .codec(IntCodec::vector(5, 0..101))
                .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
                .build();

            let result = engine.run(|ctx| ctx.score().as_i32() == EXPECTED_SUM);

            assert_eq!(result.value().iter().sum::<i32>(), EXPECTED_SUM);
            assert!(
                result.index() < BUDGET,
                "engine_can_maximize exceeded budget ({}/{BUDGET})",
                result.index()
            );
        });
    }

    #[test]
    fn engine_evolves_towards_target() {
        const TARGET: [i32; 5] = [1, 2, 3, 4, 5];
        const BUDGET: usize = 200;

        random_provider::scoped_seed(44, || {
            let mut engine = GeneticEngine::builder()
                .minimizing()
                .codec(IntCodec::vector(TARGET.len(), 0..10))
                .fitness_fn(move |geno: Vec<i32>| {
                    let mut score = 0;
                    for i in 0..geno.len() {
                        score += (geno[i] - TARGET[i]).abs();
                    }
                    score
                })
                .build();

            let result = engine.run(|ctx| ctx.score().as_i32() == 0);

            assert_eq!(result.value(), &vec![1, 2, 3, 4, 5]);
            assert!(
                result.index() < BUDGET,
                "engine_evolves_towards_target exceeded budget ({}/{BUDGET})",
                result.index()
            );
        });
    }

    #[test]
    fn engine_can_eval_batch() {
        // NOTE: This test uses `Executor::FixedSizedWorkerPool(7)` for parallel
        // batch evaluation. The fitness function itself is deterministic (no
        // RNG), and the engine's selection/alter RNG draws happen on the
        // calling thread, so `scoped_seed` still gives deterministic
        // selection behavior. If a future change moves RNG draws into the
        // parallel evaluator, this test will become flaky.
        const BUDGET: usize = 300;

        random_provider::scoped_seed(45, || {
            let mut engine = GeneticEngine::builder()
                .codec(IntChromosome::from((5, 0..100)))
                .minimizing()
                // The way the engine's workflow is setup, only individuals (phenotypes) which have been invalidated
                // (ie: those who have been mutated or crossed over) will be fed into the fitness function. Due to the
                // offspring fraction parameter being set at 0.8 by default, we'd expect a max of 80 individuals to be fed
                // into this batch fitness function, the rest will be copied down to the next generation via the survivor
                // selection. Although the real number will most likely be lower. To increase the number of individuals
                // fed into the batch we have two options:
                //   1.) Increase the offspring fraction as shown below (to 1.0) - this will cause the algorithm to completely
                //       negate the 'survivor_selector' and instead, will feed 100% of the population into the alters thus making
                //       every single phenotype open to crossover/mutation (invalidation - needing a new score).
                //          .offspring_fraction(1.0)
                //   2.) Increase the mutation/crossover rate so more individuals are invalidated during recombination.
                .executor(Executor::FixedSizedWorkerPool(7))
                .batch_fitness_fn(|phenotypes: &[Vec<i32>]| {
                    // At a very very very base level, we expect the batch to have at least two phenotypes
                    // Realistically, with an engine configured like this one is, we'd expect anywhere from 50-70ish
                    // individuals per batch here.
                    assert!(
                        phenotypes.len() > 0,
                        "Batch should have more than one phenotype"
                    );
                    phenotypes
                        .iter()
                        .map(|geno| geno.iter().sum::<i32>())
                        .collect()
                })
                .build();

            let result = engine.run(|ctx| ctx.score().as_i32() == 0);

            assert_eq!(result.value().iter().sum::<i32>(), 0);
            assert!(
                result.index() < BUDGET,
                "engine_can_eval_batch exceeded budget ({}/{BUDGET})",
                result.index()
            );
        });
    }

    #[test]
    fn test_engine_score_iterator() {
        const BUDGET: usize = 200;

        random_provider::scoped_seed(46, || {
            let engine = GeneticEngine::builder()
                .minimizing()
                .codec(IntCodec::vector(5, 0..100))
                .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
                .build();

            let result = engine
                .iter()
                .limit(Limit::Generation(BUDGET))
                .until_score(0)
                .last()
                .unwrap();

            let best = result.value();
            assert_eq!(best.iter().sum::<i32>(), 0);
            assert_eq!(result.score().as_i32(), 0);
            assert!(
                result.index() < BUDGET,
                "test_engine_score_iterator exceeded budget ({}/{BUDGET})",
                result.index()
            );
        });
    }

    #[test]
    fn test_engine_seconds_iterator() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_seconds(2_f64).last().unwrap();

        // Round here as the time taken to execute the engine may
        // be slightly over or under 2 seconds
        assert_eq!((result.time().as_secs_f64() - 2_f64).abs().round(), 0.0);
    }

    #[test]
    fn test_engine_iterations_iterator() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().limit(10).last().unwrap();
        assert_eq!(result.index(), 10);
    }

    #[test]
    fn test_engine_custom_iterator() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine
            .iter()
            .limit(vec![
                Limit::Generation(15),
                Limit::Seconds(Duration::from_secs_f64(3_f64)),
            ])
            .last()
            .unwrap();

        assert_eq!(result.index(), 15);
    }

    #[test]
    fn test_engine_control_pause_resume() {
        use std::thread;
        use std::time::Duration;

        let mut engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let control = engine.control();

        let handle = thread::spawn(move || {
            let result = engine.iter().until_seconds(1_f64).last().unwrap();
            assert_eq!((result.seconds() - 1_f64).abs().round(), 0.0);
        });

        thread::sleep(Duration::from_millis(100));
        control.set_paused(true);

        // Ensure the engine is paused for at least 500ms
        thread::sleep(Duration::from_millis(500));
        control.set_paused(false);
        handle.join().unwrap();
    }

    #[test]
    fn test_engine_control_stop() {
        use std::thread;
        use std::time::Duration;

        let mut engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let control = engine.control();

        let handle = thread::spawn(move || {
            let result = engine.iter().last().unwrap();
            assert!(result.seconds() < 5_f64);
        });

        thread::sleep(Duration::from_millis(100));
        control.stop();
        handle.join().unwrap();
    }
}
