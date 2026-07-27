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

    // There are a few different ways we can add alters to the engine in rust. Assuming you
    // use the same alters for each method below, the resulting engine will be the same.
    // Choose the one that you prefer, but keep in mind that the alters
    // will be applied in the order they are defined.

    // ---------------------------------------
    // 1.) Using the `alters!` macro - the most flexible way to add multiple
    //     mutators and crossovers
    // ---------------------------------------
    let alters = alters![GaussianMutator::new(0.1), BlendCrossover::new(0.8, 0.5),];

    let engine = GeneticEngine::builder()
        .codec(codec.clone())
        .offspring_selector(BoltzmannSelector::new(4.0))
        .survivor_selector(TournamentSelector::new(3))
        .fitness_fn(fit)
        .alter(alters) // Add the alterers to the engine
        // ... other parameters ...
        .build();

    // ---------------------------------------
    // 2.) Using `.mutator` / `.crossover` to apply a single mutator and crossover
    // ---------------------------------------
    let mutator = UniformMutator::new(0.1);
    let crossover = MultiPointCrossover::new(0.8, 2);

    let engine = GeneticEngine::builder()
        .codec(codec.clone())
        .offspring_selector(BoltzmannSelector::new(4.0))
        .survivor_selector(TournamentSelector::new(3))
        .mutator(mutator)
        .crossover(crossover)
        .fitness_fn(fit)
        // ... other parameters ...
        .build();

    // ---------------------------------------
    // 3.) Using `.mutators` / `.crossovers` with vectors of trait objects
    // ---------------------------------------
    let mutators: Vec<Box<dyn Mutate<FloatChromosome<f32>>>> = vec![
        Box::new(GaussianMutator::new(0.1)),
        Box::new(UniformMutator::new(0.05)),
    ];

    let crossovers: Vec<Box<dyn Crossover<FloatChromosome<f32>>>> = vec![
        Box::new(MultiPointCrossover::new(0.8, 2)),
        Box::new(UniformCrossover::new(0.75)),
    ];

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .survivor_selector(TournamentSelector::new(3))
        .mutators(mutators)
        .crossovers(crossovers)
        .fitness_fn(fit)
        // ... other parameters ...
        .build();

    // Run the engine: stop when score <= 0.01 or after 1000 generations
    // `.run()` here is an actual function that takes a closure which is called
    // after each generation.
    let result = engine.run(|generation| {
        let score = generation.score();
        let index = generation.index();

        score.as_f32() <= 0.01 || index >= 1000
    });
    // --8<-- [end:example]
}
