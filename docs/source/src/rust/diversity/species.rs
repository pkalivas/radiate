use radiate::prelude::*;

fn your_fitness_fn(individual: Vec<f32>) -> f32 {
    individual.iter().map(|v| v.abs()).sum()
}

fn your_char_fit_fn(individual: Vec<char>) -> usize {
    individual.iter().map(|c| *c as usize).sum()
}

fn main() {
    // --8<-- [start:threshold]
    let engine = GeneticEngine::builder()
        .codec(CharCodec::vector(10))
        .fitness_fn(your_char_fit_fn)
        // A distance measure turns speciation on; the threshold sets how close
        // two individuals must be (per the measure) to share a species.
        .diversity(HammingDistance)
        .species_threshold(0.5)
        // ... other parameters ...
        .build();
    // --8<-- [end:threshold]

    // --8<-- [start:dynamic_threshold]
    // `species_threshold` accepts a `Rate`, so it can change over generations.
    // Here it widens from 0.3 to 0.9 across the first 100 generations: start
    // fine-grained (many small species), then coarsen to encourage convergence.
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0))
        .fitness_fn(your_fitness_fn)
        .diversity(EuclideanDistance)
        .species_threshold(Rate::Linear(0.3, 0.9, 100))
        // ... other parameters ...
        .build();
    // --8<-- [end:dynamic_threshold]

    // --8<-- [start:age]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0))
        .fitness_fn(your_fitness_fn)
        .diversity(EuclideanDistance)
        .species_threshold(0.5)
        // A species that survives this many generations without improving its best
        // score is culled, and its members sit out crossover/mutation that generation.
        .max_species_age(25)
        // ... other parameters ...
        .build();
    // --8<-- [end:age]

    // --8<-- [start:target_species_count]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0))
        .fitness_fn(your_fitness_fn)
        .diversity(EuclideanDistance)
        // Instead of a distance threshold, you can specify a target number of species.
        .target_species(4)
        // ... other parameters ...
        .build();

    // Note that this is exactly the same as setting the species_threshold to an expression like so:
    let count = 4;
    let curr_threshold = 0.5; // This would be the initial threshold if we were to use a static threshold instead of target_species_count.

    let species_threshold = Expr::when(Expr::select(metric_names::INDEX).lt(2))
        .then(curr_threshold)
        .otherwise(
            (Expr::select(metric_names::SPECIES_COUNT).error(count as f32) * 0.05)
                + Expr::select(metric_names::SPECIES_THRESHOLD),
        );
    // --8<-- [end:target_species_count]
}
