use radiate::prelude::*;

// Example multi-objective components: each takes the decoded genotype and
// returns one objective's value. Defined here so the snippets below stay short.
fn objective1(genotype: &[f32]) -> f32 {
    genotype.iter().sum::<f32>()
}

fn objective2(genotype: &[f32]) -> f32 {
    genotype.iter().map(|g| g * g).sum::<f32>()
}

fn main() {
    // --8<-- [start:minimizing]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0)) // Example codec
        .minimizing() // Configure for minimization
        .fitness_fn(|genotype: Vec<f32>| {
            // Return a value to minimize
            genotype.iter().sum::<f32>()
        })
        // ... other parameters ...
        .build();
    // --8<-- [end:minimizing]

    // --8<-- [start:maximizing]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0)) // Example codec
        .maximizing() // Configure for maximization
        .fitness_fn(|genotype: Vec<f32>| {
            // Return a value to maximize
            genotype.iter().sum::<f32>()
        })
        // ... other parameters ...
        .build();
    // --8<-- [end:maximizing]

    // --8<-- [start:multi_objective]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0)) // Example codec
        .multi_objective(vec![Optimize::Minimize, Optimize::Maximize])
        .front_size(800..900) // Pareto front size range
        .fitness_fn(|genotype: Vec<f32>| {
            // Return a vector of fitness values
            vec![
                objective1(&genotype), // Minimize this
                objective2(&genotype), // Maximize this
            ]
        })
        // ... other parameters ...
        .build();
    // --8<-- [end:multi_objective]

    // --8<-- [start:multi_objective_selectors]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0)) // Example codec
        .multi_objective(vec![Optimize::Minimize, Optimize::Maximize])
        // Tournament selection with Pareto dominance
        .offspring_selector(TournamentNSGA2Selector::new())
        // NSGA-III with 12 reference directions
        .survivor_selector(NSGA3Selector::new(12))
        .front_size(800..900) // Pareto front size range
        .fitness_fn(|genotype: Vec<f32>| {
            vec![
                objective1(&genotype), // Minimize this
                objective2(&genotype), // Maximize this
            ]
        })
        .build();
    // --8<-- [end:multi_objective_selectors]
}
