use radiate::prelude::*;
use std::collections::VecDeque;
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
    // --8<-- [start:limits_score]
    // Run until a score target is reached
    let engine = build_engine();
    let target_score = 0.01;
    let result = engine.iter().until_score(target_score).last().unwrap();
    // --8<-- [end:limits_score]

    // --8<-- [start:limits_seconds]
    // Run until a time limit is reached
    let engine = build_engine();
    let time_limit = Duration::from_secs(60);
    let result = engine.iter().until_duration(time_limit).last().unwrap();
    // --8<-- [end:limits_seconds]

    // --8<-- [start:limits_convergence]
    // Run until the score stops improving by more than `epsilon` over a sliding `window`
    let engine = build_engine();
    let window = 50;
    let epsilon = 0.01; // how close the scores must be over the window to consider convergence
    let result = engine
        .iter()
        .limit(Limit::Convergence(
            window,
            epsilon,
            VecDeque::with_capacity(window),
        ))
        .last()
        .unwrap();
    // --8<-- [end:limits_convergence]

    // --8<-- [start:limits_combined]
    // Combine several limits - the engine stops as soon as ANY one of them is reached
    let engine = build_engine();
    let result = engine
        .iter()
        .logging()
        .limit((
            Limit::Generation(100),
            Limit::Seconds(Duration::from_secs_f64(2.0)),
            Limit::Score(0.01.into()),
        ))
        .last()
        .unwrap();
    // --8<-- [end:limits_combined]
}
