# Example

Lets add on to our example - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll keep the previous inputs the same as before, but now we add `diversity` to the `GeneticEngine`.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/diversity/example.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Define a fitness function that uses the decoded values
    fn fitness_fn(individual: Vec<f32>) -> f32 {
        let a = individual[0];
        let b = individual[1];
        calculate_error(a, b)  // Your error calculation here
    }

    // This will produce a Genotype<FloatChromosome> with 1 FloatChromosome which
    // holds 2 FloatGenes (a and b), each with a value between -1.0 and 1.0 and a bound between -10.0 and 10.0
    let codec = FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0);

    // Use Boltzmann selection for offspring - individuals which
    // will be used to create new individuals through mutation and crossover
    let offspring_selector = BoltzmannSelector::new(4.0);

    // Use tournament selection for survivors - individuals which will
    // be passed down unchanged to the next generation
    let survivor_selector = TournamentSelector::new(3);

    // Define some alters 
	let alters = alters![
		GaussianMutator::new(0.1),
		BlendCrossover::new(0.8, 0.5)
	];

    // Define the diversity measure
    let diversity = HammingDistance::new(); // or EuclideanDistance::new() for continuous problems

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(offspring_selector)
        .survivor_selector(survivor_selector)
        .fitness_fn(fitness_fn)
		.alterers(alters) 
        .diversity(diversity)       // Add the diversity measure
        .species_threshold(0.5)     // Default value
        .max_species_age(20)        // Default value
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| {
        // Now because we have added diversity, the ecosystem will include species like such:
        let species = generation.species().unwrap();
        println!("Species count: {}", species.len());
        generation.index() >= 1000 || generation.score().as_f32() <= 0.01
    });
    ```

