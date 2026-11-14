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

    // Use Boltzmann selection for offspring - individuals which
    // will be used to create new individuals through mutation and crossover
    let offspring_selector = BoltzmannSelector::new(4.0);

    // Use tournament selection for survivors - individuals which will
    // be passed down unchanged to the next generation
    let survivor_selector = TournamentSelector::new(3);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(offspring_selector)
        .survivor_selector(survivor_selector)
        .fitness_fn(fit)
        // ... other parameters ...
        .build();

    // Run the engine: stop after 1000 generations
    // `.run()` is equal to `.last().unwrap()`
    let result = engine.iter().take(1000).run();
    // --8<-- [end:example]
}
