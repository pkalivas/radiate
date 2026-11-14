use radiate::prelude::*;

const MIN_SCORE: i32 = 100;

fn main() {
    // --8<-- [start:ui]
    let engine = GeneticEngine::builder()
        // ... configure your engine as normal ...
        .codec(IntCodec::vector(5, 0..100))
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    // Wrap the engine with the UI
    let final_generation = radiate::ui(engine).iter().take(10).last().unwrap();
    // --8<-- [end:ui]
}
