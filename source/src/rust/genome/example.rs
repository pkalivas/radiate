use radiate::prelude::*;

fn calculate_error(a: f32, b: f32) -> f32 {
    // Your error calculation here
    (a - 1.0).powi(2) + (b - 2.0).powi(2) // Example: minimize distance from (1.0, 2.0)
}

fn main() {
    // --8<-- [start:codec_example]
    // Define a fitness function that uses the decoded values
    fn fit(individual: Vec<f32>) -> f32 {
        let a = individual[0];
        let b = individual[1];
        calculate_error(a, b) // Your error calculation here
    }

    // This will produce a Genotype<FloatChromosome> with 1 FloatChromosome which
    // holds 2 FloatGenes (a and b), each with a value between -1.0 and 1.0 and a bound between -10.0 and 10.0
    let codec = FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(fit)
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations
    let result = engine.iter().take(100).last().unwrap();
    // --8<-- [end:codec_example]

    // --8<-- [start:chromosome_codec_example]
    // To create a matrix codec using a Chromosome just use a Vec
    let engine = GeneticEngine::builder()
        .codec(vec![
            FloatChromosome::from((2, -1.0..1.0, -10.0..10.0)),
            FloatChromosome::from(vec![
                FloatGene::from(-3.0..3.0),
                FloatGene::from((-5.0..5.0, -10.0..10.0)),
            ]),
        ])
        .fitness_fn(|phenotype: Vec<Vec<f32>>| {
            // ... your fitness calc ...
            return 0;
        })
        // ... other parameters ...
        .build();
    // --8<-- [end:chromosome_codec_example]
}
