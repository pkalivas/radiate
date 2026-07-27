use radiate::prelude::*;
use std::sync::Arc;
use std::time::Duration;

fn my_fitness_fn(genotype: f32) -> f32 {
    genotype // placeholder: your real fitness logic here
}

// Each example below starts from a freshly built engine.
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
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype)) // Return a single fitness score
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations - the result will be a `Generation<FloatChromosome<f32>, f32>`
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
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .multi_objective(vec![Optimize::Minimize, Optimize::Maximize]) // Specify multi-objective optimization
        // Return a multi-objective fitness score (one value per objective)
        .fitness_fn(|genotype: f32| vec![my_fitness_fn(genotype), my_fitness_fn(genotype)])
        // ... other parameters ...
        .build();

    // Run the engine for 100 generations
    let result = engine.iter().take(100).last().unwrap();

    // Everything in this generation is the same as the single-objective epoch, except that
    // the call to `front()` will return the Pareto `Front`:
    // This will be of type `Front<Phenotype<FloatChromosome<f32>>>`
    let front: &Front<Phenotype<FloatChromosome<f32>>> = result.front().unwrap();

    // Get the members of the Pareto front:
    let individuals: &[Arc<Phenotype<FloatChromosome<f32>>>] = front.values();
    // --8<-- [end:multi_objective]

    // --8<-- [start:generation_view]
    // `until` takes a closure over a borrowed `GenerationView` - no `Generation` clone
    // happens on any generation except (implicitly) the last one, when `.last()` needs
    // to hand back an owned result.
    let engine = build_engine();
    let result = engine
        .iter()
        .until(|view: GenerationView<FloatChromosome<f32>, f32>| view.score().as_f32() <= 0.01)
        .last()
        .unwrap();
    // --8<-- [end:generation_view]
}
