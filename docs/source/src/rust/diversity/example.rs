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

    // Define the diversity measure
    let diversity = EuclideanDistance; // or HammingDistance for discrete problems

    let engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .survivor_selector(TournamentSelector::new(3))
        .fitness_fn(fit)
        .alter(alters!(
            GaussianMutator::new(0.1),
            BlendCrossover::new(0.8, 0.5)
        ))
        .diversity(diversity) // Add the diversity measure
        .species_threshold(0.5) // Default value
        .max_species_age(25) // Default value
        // ... other parameters ...
        .build();

    // Run the engine: stop after 100 generations.
    // Now that we've added diversity, each generation's ecosystem includes species:
    let result = engine
        .iter()
        .inspect(|generation| {
            let species = generation.species().unwrap();
            println!("Species count: {}", species.len());
        })
        .take(100)
        .run();
    // --8<-- [end:example]
}
