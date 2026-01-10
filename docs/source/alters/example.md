# Example

Continuing with our example from the previous two sections - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll use the same `codec` and `fitness_function` as before, but this time we'll add alterers to the `GeneticEngine` to evolve the parameters.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Define a fitness function that uses the decoded values
    def fitness_function(individual: list[float]) -> float:    
        # Calculate how well these parameters fit your data
        a = individual[0]
        b = individual[1]
        return calculate_error(a, b)  # Your error calculation here

    # Create a codec for two parameters (a and b)
    codec = rd.FloatCodec.vector(
        length=2,                   # We need two parameters: a and b
        init_range=(-1.0, 1.0),    # Start with values between -1 and 1
        bounds=(-10.0, 10.0)       # Allow evolution to modify the values between -10 and 10
    )

    # Use Boltzmann selection for offspring - individuals which
    # will be used to create new individuals through mutation and crossover
    offspring_selector = rd.BoltzmannSelector(temp=4)

    # Use tournament selection for survivors - individuals which will 
    # be passed down unchanged to the next generation
    survivor_selector = rd.TournamentSelector(k=3)

	# Define the alterers - these will be applied to the selected offspring
	# to create new individuals. They will be applied in the order they are defined.
	alters = [
		rd.GaussianMutator(rate=0.1),
		rd.BlendCrossover(rate=0.8, alpha=0.5)
	]

    # Create the evolution engine
    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_function,
        offspring_selector=offspring_selector,
        survivor_selector=survivor_selector,
		alters=alters # Add the alterers to the engine
        # ... other parameters ...
    )

    # Run the engine
    result = engine.run([rd.ScoreLimit(0.01), rd.GenerationsLimit(1000)])
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

	// There are a few different ways we can add alters to the engine in rust. Assuming you 
	// use the same alters for each method below, the resulting engine will be the same.
	// Choose the one that you prefer, but keep in mind that the alters 
	// will be applied in the order they are defined.

	// ---------------------------------------
	// 1.) Using the "alters!" macro - this is the most flexible way to add multiple mutators and crossovers
	// ---------------------------------------
	let alters = alters![
		GaussianMutator::new(0.1),
		BlendCrossover::new(0.8, 0.5)
	];

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(offspring_selector)
        .survivor_selector(survivor_selector)
        .fitness_fn(fitness_fn)
		.alterers(alters) // Add the alterers to the engine
        // ... other parameters ...
        .build();

	// ---------------------------------------
	// 2.) Using the "mutators" and "crossovers" methods to apply a single mutator and crossover
	// ---------------------------------------
	let mutator = UniformMutator::new(0.1);
	let crossover = MultiPointCrossover::new(0.8, 2);

	let mut engine = GeneticEngine::builder()
		.codec(codec)
		.offspring_selector(offspring_selector)
		.survivor_selector(survivor_selector)
		.mutator(mutator)
		.crossover(crossover)
		.fitness_fn(fitness_fn)
		// ... other parameters ...
		.build();

	// ---------------------------------------
	// 3.) Using the "mutators" and "crossovers" methods with vectors
	// ---------------------------------------
	let mutators: Vec<Box<dyn Mutator>> = vec![
		Box::new(GaussianMutator::new(0.1)),
		Box::new(UniformMutator::new(0.05)),
	];

	let crossovers: Vec<Box<dyn Crossover>> = vec![
		Box::new(MultiPointCrossover::new(0.8, 2)),
		Box::new(UniformCrossover::new(0.75)),
	];

	let mut engine = GeneticEngine::builder()
		.codec(codec)
		.offspring_selector(offspring_selector)
		.survivor_selector(survivor_selector)
		.mutators(mutators)
		.crossovers(crossovers)
		.fitness_fn(fitness_fn)
		// ... other parameters ...
		.build();

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 1000 || generation.score().as_f32() <= 0.01
    });
    ```
