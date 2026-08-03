#[cfg(test)]
mod engine_tests {
    use radiate_core::*;
    use radiate_engines::*;
    use radiate_test::*;
    use rstest::*;
    use std::time::Duration;

    #[test]
    fn engine_can_minimize() {
        const EXPECTED_SUM: i32 = 0;
        const EXPECTED_GENS: usize = 60;

        seeded(42, || {
            let result = int_minimize_engine(5, 100)
                .iter()
                .until_score(0)
                .last()
                .unwrap();

            let best = result.value();

            assert_eq!(best.iter().sum::<i32>(), EXPECTED_SUM);
            assert_eq!(result.index(), EXPECTED_GENS);
        });
    }

    #[test]
    fn engine_can_maximize() {
        const EXPECTED_SUM: usize = 20;
        const BUDGET: usize = 500;

        seeded(43, || {
            let engine = onemax_engine(20);
            let result = engine
                .run(|ctx| ctx.score().as_usize() == EXPECTED_SUM)
                .unwrap();

            assert_eq!(result.value().iter().filter(|&&x| x).count(), EXPECTED_SUM);
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

        seeded(44, || {
            let engine = GeneticEngine::builder()
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

            let result = engine.run(|ctx| ctx.score().as_i32() == 0).unwrap();

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

        seeded(45, || {
            let engine = GeneticEngine::builder()
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

            let result = engine.run(|ctx| ctx.score().as_i32() == 0).unwrap();

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

        seeded(46, || {
            let result = int_minimize_engine(10, 100)
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
        let result = onemax_engine(200)
            .iter()
            .until_seconds(2_f64)
            .last()
            .unwrap();

        // Round here as the time taken to execute the engine may
        // be slightly over or under 2 seconds
        assert_eq!((result.time().as_secs_f64() - 2_f64).abs().round(), 0.0);
    }

    #[test]
    fn test_engine_iterations_iterator() {
        let result = onemax_engine(50).iter().limit(10).last().unwrap();
        assert_eq!(result.index(), 10);
    }

    #[test]
    fn test_engine_custom_iterator() {
        let engine = onemax_engine(5);

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

        let mut engine = onemax_engine(5);
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
    fn diag_test_engine_control_step_n() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;
        use std::time::Duration;

        let mut engine = onemax_engine(5);
        let control = engine.control();
        control.step_n(10);

        let last_index = Arc::new(AtomicUsize::new(0));
        let last_index2 = Arc::clone(&last_index);

        let handle = thread::spawn(move || {
            let mut runtime = engine.iter().limit(1000);
            while let Some(epoch) = runtime.next() {
                last_index2.store(epoch.index(), Ordering::SeqCst);
            }
        });

        thread::sleep(Duration::from_millis(500));
        let observed_at_500ms = last_index.load(Ordering::SeqCst);
        println!("index observed 500ms after step_n(10), before forcing stop: {observed_at_500ms}");

        control.stop();
        handle.join().unwrap();
        println!(
            "final index after forced stop: {}",
            last_index.load(Ordering::SeqCst)
        );
    }

    #[test]
    fn test_engine_control_stop() {
        use std::thread;
        use std::time::Duration;

        let mut engine = onemax_engine(5);
        let control = engine.control();

        let handle = thread::spawn(move || {
            let result = engine.iter().last().unwrap();
            assert!(result.seconds() < 5_f64);
        });

        thread::sleep(Duration::from_millis(100));
        control.stop();
        handle.join().unwrap();
    }

    #[test]
    fn subscribers_registered_on_the_builder_actually_fire() {
        // Regression test: `EngineConfig::from` used to build a fresh, empty
        // `ActorSystem::default()` instead of carrying over
        // `EngineParams::event_system` — every `.on_start()`/
        // `.on_epoch_complete()`/`.subscribe()` registered during the
        // builder chain was silently discarded at `try_build()` time, so no
        // handler ever fired on a real, built engine (only on an
        // `ActorSystem` constructed and driven directly, which is why the
        // unit tests in `events::message` didn't catch it).
        let collector = MetricCollector::new();
        let history = collector.history();

        let engine = GeneticEngine::builder()
            .problem(OneMax::new(5))
            .on_epoch_complete(collector)
            .build();

        const BUDGET: usize = 10;
        let _ = engine.iter().limit(BUDGET).last().unwrap();

        assert_eq!(history.lock().unwrap().len(), BUDGET);
    }

    #[test]
    fn metric_step_publishes_stagnation_warning() {
        use radiate_engines::events::LogWarn;
        use std::sync::{Arc, Mutex};

        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen2 = Arc::clone(&seen);

        // A fitness function that ignores the genotype entirely means every
        // individual scores identically forever — the first generation sets
        // the "best" (an improvement over nothing), and every generation
        // after that is guaranteed stagnant, so this deterministically
        // crosses the warning threshold rather than depending on chance.
        let engine = GeneticEngine::builder()
            .codec(FloatCodec::vector(4, -5.0..5.0))
            .fitness_fn(|_geno: Vec<f32>| 1.0)
            .subscribe_typed::<LogWarn, _>(move |w: LogWarn, _ctx: &EventContext| {
                seen2.lock().unwrap().push(w.0);
            })
            .build();

        const BUDGET: usize = 60;
        let _ = engine.iter().limit(BUDGET).last().unwrap();

        let warnings = seen.lock().unwrap();
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one stagnation warning, got {:?}",
            *warnings
        );
        assert!(warnings[0].contains("no improvement"));
    }

    #[test]
    fn metric_step_publishes_actor_system_health() {
        // `on_epoch_complete` subscribes an actor to `EpochCompleted<T>`,
        // which the engine actually dispatches every single generation
        // (unlike `Warn`, which only fires past the stagnation threshold) —
        // that's what gives this test real processed-message traffic to
        // observe within a small budget.
        let engine = GeneticEngine::builder()
            .codec(FloatCodec::vector(4, -5.0..5.0))
            .fitness_fn(|_geno: Vec<f32>| 1.0)
            .on_epoch_complete(|_event: EpochCompleted<Vec<f32>>, _ctx: &EventContext| {})
            .build();

        const BUDGET: usize = 5;
        let result = engine.iter().limit(BUDGET).last().unwrap();
        let metrics = result.metrics();

        let subscriptions = metrics
            .get(metric_names::ACTOR_SUBSCRIPTIONS)
            .expect("actor.subscriptions metric should be present")
            .last_value();
        assert!(
            subscriptions >= 1.0,
            "expected at least the EpochCompleted subscription registered, got {subscriptions}"
        );

        let queued = metrics
            .get(metric_names::ACTOR_QUEUED)
            .expect("actor.queued metric should be present")
            .last_value();
        assert_eq!(queued, 0.0, "mailbox should be fully drained between generations");

        // The metric snapshot for a generation is taken mid-pipeline, before
        // that same generation's own `EpochEnd` dispatch — so this lags one
        // generation behind, but should still be solidly positive by the
        // end of a 5-generation run.
        let processed = metrics
            .get(metric_names::ACTOR_PROCESSED)
            .expect("actor.processed metric should be present")
            .last_value();
        assert!(
            processed > 0.0,
            "expected at least one EpochCompleted event to have been processed, got {processed}"
        );
    }

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
            let test_inputs = (-10..=10).map(|i| i as f32 / 5.0).collect::<Vec<f32>>();
            let targets = test_inputs
                .iter()
                .map(|x| 2.0 * x + 1.0)
                .collect::<Vec<f32>>();

            fn fitness(coeffs: &Vec<f32>, inputs: &[f32], targets: &[f32]) -> f32 {
                inputs
                    .iter()
                    .zip(targets.iter())
                    .map(|(x, y)| {
                        let pred = coeffs[0] * x + coeffs[1];
                        (pred - y).powi(2)
                    })
                    .sum::<f32>()
                    / inputs.len() as f32
            }

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
                .fitness_fn(move |geno: Vec<f32>| fitness(&geno, &test_inputs, &targets))
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

            assert_population_speciated(result.ecosystem(), "speciated population integrity");
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

            assert_population_speciated(result.ecosystem(), "speciated small population");
            assert_eq!(result.population().len(), POP_SIZE);
        });
    }

    /// As a speciated population converges toward the sphere optimum, every
    /// individual crowds into the same neighborhood and species collapse
    /// down to one — SpeciateStep should publish exactly the kind of
    /// diversity-collapse warning this whole event was designed to surface.
    #[test]
    fn speciate_step_publishes_diversity_collapse_warning() {
        use radiate_engines::events::LogWarn;
        use std::sync::{Arc, Mutex};

        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen2 = Arc::clone(&seen);

        seeded(2024, || {
            let problem = Sphere::new(2, 10.0);
            let engine = GeneticEngine::builder()
                .problem(problem)
                .minimizing()
                .population_size(50)
                .diversity(EuclideanDistance)
                .species_threshold(0.05)
                .alter(alters![
                    BlendCrossover::new(0.5, 0.5),
                    GaussianMutator::new(0.05)
                ])
                .subscribe_typed::<LogWarn, _>(move |w: LogWarn, _ctx: &EventContext| {
                    seen2.lock().unwrap().push(w.0);
                })
                .build();

            let _ = engine.iter().limit(200).last().unwrap();
        });

        let warnings = seen.lock().unwrap();
        assert!(
            warnings.iter().any(|w| w.contains("collapsed")),
            "expected a species-collapse warning as the population converged, got: {:?}",
            *warnings
        );
    }
}
