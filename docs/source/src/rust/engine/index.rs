use radiate::prelude::*;

fn my_fitness_fn(genotype: f32) -> f32 {
    genotype // placeholder: your real fitness logic here
}

fn main() {
    // --8<-- [start:minimal_engine]
    // Only a codec and a fitness function are required - everything else uses its default.
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype))
        .build();

    let result = engine.iter().until_generation(100).last().unwrap();
    // --8<-- [end:minimal_engine]
}
