use radiate::prelude::*;

// Example multi-objective components: each takes the decoded genotype and
// returns one objective's value. Defined here so the snippets below stay short.
fn objective1(genotype: &[f32]) -> f32 {
    genotype.iter().sum::<f32>()
}

fn objective2(genotype: &[f32]) -> f32 {
    genotype.iter().map(|g| g * g).sum::<f32>()
}

// Pulls every gene toward 0.5, so it conflicts with `objective1` (which pulls toward 0.0)
// when both are minimized.
fn distance_from_half(genotype: &[f32]) -> f32 {
    genotype.iter().map(|g| (g - 0.5).powi(2)).sum::<f32>()
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
        // Crowded-comparison tournament for parents
        .offspring_selector(TournamentNSGA2Selector::new())
        // Rank + crowding distance for survivors
        .survivor_selector(NSGA2Selector::new())
        // Keep more of the already-evaluated front each generation
        .offspring_fraction(0.5)
        .alter(alters!(
            SimulatedBinaryCrossover::new(0.8, 20.0),
            PolynomialMutator::new(0.1, 20.0),
        ))
        .front_size(800..900) // Pareto front size range
        .fitness_fn(|genotype: Vec<f32>| {
            vec![
                objective1(&genotype), // Minimize this
                objective2(&genotype), // Maximize this
            ]
        })
        .build();
    // --8<-- [end:multi_objective_selectors]

    // --8<-- [start:front_reading]
    let codec = FloatCodec::vector(10, 0.0..1.0);

    let engine = GeneticEngine::builder()
        .codec(codec.clone())
        .multi_objective(vec![Optimize::Minimize, Optimize::Minimize])
        .front_size(800..900)
        .fitness_fn(|genotype: Vec<f32>| {
            vec![objective1(&genotype), distance_from_half(&genotype)]
        })
        .build();

    let result = engine.iter().take(200).last().unwrap();

    // The archive of the best trade-offs found over the whole run
    let front = result.front().expect("multi-objective runs always have a front");
    println!("{} solutions on the front", front.len());

    for member in front.values() {
        let scores = member.score().unwrap().as_slice(); // one value per objective
        let params: Vec<f32> = codec.decode(member.genotype()); // back to the problem's representation
        println!("{scores:?} <- {params:?}");
    }
    // --8<-- [end:front_reading]

    // --8<-- [start:front_analysis]
    // The analysis methods cache their results, so they need an owned, mutable front
    let mut front = front.clone();

    // Reference point: worse than every member on every objective
    let hypervolume = front.hypervolume(&[10.0, 3.0]);
    let entropy = front.entropy();
    let crowding = front.crowding_distance().map(|d| d.to_vec());

    // Drop the 2% most isolated members
    let removed = front.remove_outliers(0.02);
    // --8<-- [end:front_analysis]
}
