use radiate::prelude::*;

fn calculate_error(a: f32, b: f32) -> f32 {
    // Your error calculation here
    (a - 1.0).powi(2) + (b - 2.0).powi(2) // Example: minimize distance from (1.0, 2.0)
}

fn main() {
    // --8<-- [start:example]
    // Define a fitness function that uses the decoded values
    fn fit(individual: Vec<f32>) -> f32 {
        let a = individual[0];
        let b = individual[1];
        calculate_error(a, b) // Your error calculation here
    }

    // This will produce a Genotype<FloatChromosome> with 1 FloatChromosome which
    // holds 2 FloatGenes (a and b), each with a value between -1.0 and 1.0 and a bound between -10.0 and 10.0
    let codec = FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0);

    // Define the executor - here we use a fixed size worker pool with 4 threads
    let executor = Executor::FixedSizedWorkerPool(4);
    // Alternatively, you can use a WorkerPool (which uses rayon's global thread pool).
    // Requires the `rayon` feature to be enabled in your Cargo.toml.
    let executor = Executor::WorkerPool;
    // Or for single-threaded execution, use Serial - this is the default if none is specified
    let executor = Executor::Serial;

    let engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .survivor_selector(TournamentSelector::new(3))
        .fitness_fn(fit)
        .alter(alters!(
            GaussianMutator::new(0.1),
            BlendCrossover::new(0.8, 0.5),
        ))
        .executor(executor) // Set the executor here
        // ... other parameters ...
        .build();

    // Run the engine: stop after 1000 generations
    let result = engine.iter().take(1000).run();
    // --8<-- [end:example]

    // --8<-- [start:parallel]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0))
        .fitness_fn(fit)
        // ... other builder methods ...
        .parallel()
        .build();
    // --8<-- [end:parallel]
}
