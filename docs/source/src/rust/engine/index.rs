use radiate::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn my_fitness_fn(genotype: f32) -> f32 {
    genotype // placeholder: your real fitness logic here
}

// `iter()` consumes the engine, so the "Running" examples below each start from a
// freshly built engine. This helper keeps those snippets short.
fn build_engine() -> GeneticEngine<FloatChromosome<f32>, f32> {
    GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype))
        .build()
}

fn main() {
    // --8<-- [start:single_objective]
    // Create an engine of type:
    // `GeneticEngine<FloatChromosome<f32>, f32>`
    //
    // Where the `epoch` is `Generation<FloatChromosome<f32>, f32>`
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype)) // Return a single fitness score
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations - the result will be a `Generation<FloatChromosome<f32>, f32>`
    let result =
        engine.run(|generation: &Generation<FloatChromosome<f32>, f32>| generation.index() >= 100);

    // -- or using the engine's iterator --
    let result = engine.iter().take(100).last().unwrap();

    // Get the best individual's decoded value:
    let best_value: &f32 = result.value();

    // Get the score (fitness) of the best individual (or epoch score):
    let best_score: &Score = result.score();

    // Get the index of the epoch (number of generations):
    let index: usize = result.index();

    // Get the ecosystem level information:
    let ecosystem: &Ecosystem<FloatChromosome<f32>> = result.ecosystem();
    let population: &Population<FloatChromosome<f32>> = ecosystem.population();
    let species: Option<&Vec<Species<FloatChromosome<f32>>>> = ecosystem.species();

    // Get performance metrics:
    let metrics: &MetricSet = result.metrics();

    // Get evolution duration (also available in metrics):
    let time: Duration = result.time();

    // Get the objective of the engine
    let objective: &Objective = result.objective();
    // --8<-- [end:single_objective]

    // --8<-- [start:multi_objective]
    // Create an engine of type:
    // `GeneticEngine<FloatChromosome<f32>, f32>`
    //
    // Where the `epoch` is `Generation<FloatChromosome<f32>, f32>`
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .multi_objective(vec![Optimize::Minimize, Optimize::Maximize]) // Specify multi-objective optimization
        // Return a multi-objective fitness score (one value per objective)
        .fitness_fn(|genotype: f32| vec![my_fitness_fn(genotype), my_fitness_fn(genotype)])
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations
    let result =
        engine.run(|generation: &Generation<FloatChromosome<f32>, f32>| generation.index() >= 100);

    // -- or using the engine's iterator --
    let result = engine.iter().take(100).last().unwrap();

    // Everything in this generation is the same as the single-objective epoch, except that
    // the call to `front()` will return the Pareto `Front`:
    // This will be of type `Front<Phenotype<FloatChromosome<f32>>>`
    let front: &Front<Phenotype<FloatChromosome<f32>>> = result.front().unwrap();

    // Get the members of the Pareto front:
    let individuals: &[Arc<Phenotype<FloatChromosome<f32>>>] = front.values();
    // --8<-- [end:multi_objective]

    // --8<-- [start:running]
    // 1.) use a simple for loop to iterate through 100 generations
    let engine = build_engine();
    for epoch in engine.iter().take(100) {
        println!(
            "Generation {}: Score = {}",
            epoch.index(),
            epoch.score().as_f32()
        );
    }

    // 2.) use the iterator's custom methods to run until a score target is reached
    let engine = build_engine();
    let target_score = 0.01;
    let result = engine.iter().until_score(target_score).last().unwrap();

    // 3.) run until a time limit is reached
    let engine = build_engine();
    let time_limit = Duration::from_secs(60);
    let result = engine.iter().until_duration(time_limit).last().unwrap();

    // 4.) run until convergence
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

    // 5.) log the progress of the engine to the console using the `logging()` method
    let engine = build_engine();
    let result = engine.iter().logging().until_seconds(10.0).last().unwrap();

    // 6.) combined limits
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

    // 7.) metrics limit - stop after 1000 evaluations
    let engine = build_engine();
    let result = engine
        .iter()
        .until_metric(
            &metric_names::EVALUATION_COUNT,
            Arc::new(|metric| metric.sum() >= 1000.0),
        )
        .last()
        .unwrap();

    // 8.) Checkpointing - save the engine state every 10 generations
    let engine = build_engine();
    let checkpoint_path = "checkpoint.json";
    let result = engine
        .iter()
        .checkpoint(10, checkpoint_path)
        .take(100)
        .last()
        .unwrap();

    // 9.) Using the engine's run method with a closure - stop after 100 generations
    let mut engine = build_engine();
    let result =
        engine.run(|generation: &Generation<FloatChromosome<f32>, f32>| generation.index() >= 100);
    // --8<-- [end:running]

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
