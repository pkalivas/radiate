use radiate::prelude::*;

fn your_fitness_fn(individual: Vec<f32>) -> f32 {
    individual.iter().map(|v| v.abs()).sum()
}

fn main() {
    // Setup (outside the marker so the rendered snippet stays focused on the
    // diversity wiring): a plain codec for the engine to evolve.
    let your_codec = FloatCodec::vector(2, -1.0..1.0);

    // --8<-- [start:diversity_basic]
    let engine = GeneticEngine::builder()
        .codec(your_codec)
        .fitness_fn(your_fitness_fn)
        // A distance measure turns speciation on; the threshold sets how close
        // two individuals must be (per the measure) to share a species.
        .diversity(EuclideanDistance)
        .species_threshold(0.5)
        // ... other parameters ...
        .build();
    // --8<-- [end:diversity_basic]
}
