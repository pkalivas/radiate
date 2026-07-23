use radiate::prelude::*;
use std::thread;
use std::time::Duration;

fn my_fitness_fn(genotype: f32) -> f32 {
    genotype // placeholder: your real fitness logic here
}

// `iter()` consumes the engine, so each example below starts from a freshly built one.
fn build_engine() -> GeneticEngine<FloatChromosome<f32>, f32> {
    GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype))
        .build()
}

fn main() {
    // --8<-- [start:iterator_basic]
    // `iter()` consumes the engine and hands back an `EngineRuntime`, which implements
    // the standard `Iterator` trait - so a plain for loop works directly. `take(n)` here
    // is `EngineRuntime`'s own method (an alias for `until_generation(n)`), not
    // `std::iter::Iterator::take` - it appends a `Limit::Generation(n)` to the runtime's
    // existing limits and returns the same runtime, rather than wrapping it in a
    // `std::iter::Take<Self>`, but reads the same in a for loop either way.
    let engine = build_engine();
    for epoch in engine.iter().take(100) {
        println!(
            "Generation {}: Score = {}",
            epoch.index(),
            epoch.score().as_f32()
        );
    }
    // --8<-- [end:iterator_basic]

    // --8<-- [start:iterator_run]
    // This closure needs a `&Generation` to decide whether to stop, so `run()` here
    // builds one on every single generation (via `Engine::next()` = step() + epoch()) -
    // the same cost as iterating, even though no iterator is involved.
    let mut engine = build_engine();
    let result =
        engine.run(|generation: &Generation<FloatChromosome<f32>, f32>| generation.index() >= 100);

    // `take(100)` attaches a `Limit::Generation(100)` instead; `.last()` here is
    // `EngineRuntime`'s own inherent method (not `Iterator::last()`), so it never calls
    // `Iterator::next()` at all - `epoch()` only runs once, after the limit trips.
    let engine = build_engine();
    let result = engine.iter().take(100).last().unwrap();
    // --8<-- [end:iterator_run]

    // --8<-- [start:run_convenience]
    // The common case: attach a `Limit`, run to completion, get the final epoch. This
    // stays on the cheap path - no `Generation` is built until the limit trips.
    let engine = build_engine();
    let result = engine.iter().until_generation(100).run().unwrap();
    // --8<-- [end:run_convenience]

    // --8<-- [start:iterator_actions]
    // `logging()` (or `log_every(n)` to throttle it) prints progress to the console
    // every generation; `checkpoint(interval, path)` persists engine state periodically.
    let engine = build_engine();
    let result = engine.iter().logging().until_seconds(10.0).last().unwrap();

    let engine = build_engine();
    let checkpoint_path = "checkpoint.json";
    let result = engine
        .iter()
        .checkpoint(10, checkpoint_path)
        .take(100)
        .last()
        .unwrap();
    // --8<-- [end:iterator_actions]

    // --8<-- [start:control]
    let mut engine = GeneticEngine::builder()
        .minimizing()
        .codec(IntCodec::vector(5, 0..100))
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    let control = engine.control();

    let handle = thread::spawn(move || {
        // Run the engine for 1 second
        let result = engine.iter().until_seconds(1_f64).last().unwrap();
        // because we are running for only a second and are pausing the engine,
        // the engine's internal time tracking should be very close to 1 second even
        // though we paused it for +500ms
        assert_eq!((result.seconds() - 1_f64).abs().round(), 0.0);
    });

    thread::sleep(Duration::from_millis(100));
    control.set_paused(true);

    // Ensure the engine is paused for at least 500ms
    thread::sleep(Duration::from_millis(500));
    control.set_paused(false);
    handle.join().unwrap();
    // --8<-- [end:control]
}
