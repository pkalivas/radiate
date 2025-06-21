
# Diversity

Diversity is an opt-in aspect of radiate's genetic algorithm. At its core, this operator helps maintain a healthy population and prevent premature convergence. By using this operator, the `GeneticEngine` will split the `population` up into `species` by measuring the genetic distance, or diversity, between individuals during the evolution process. Its important to note that adding a diversity operator will increase the computational cost of the algorithm, so it should be used judiciously based on the problem at hand.

---

## Overview

Diversity in Radiate is implemented through two main components:

1. **Diversity Measurement**: Methods to quantify how different individuals are from each other. This is typically done by measuring the genetic distance between individuals - meaning the actual `chromosomes` and `genes` that make up the individuals.
2. **Species Management**: Mechanisms to group similar individuals and maintain population diversity

---

### Hamming Distance

**Compatible with**: `FloatGene`, `IntGene<I>`, `BitGene`, `CharGene`, `PermuationGene<A>`

The Hamming Distance measures diversity by counting the number of positions at which corresponding genes are different, normalized by the total number of genes. This is particularly useful for:

- Binary or discrete genetic representations
- Problems where exact matches are important
- Cases where you want to measure diversity based on exact gene differences

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	diversity = rd.HammingDistance()
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let diversity = HammingDistance::new();
	```

---

### Euclidean Distance

**Compatible with**: `FloatGene`, `IntGene<I>`

The Euclidean Distance calculates the square root of the sum of squared differences between corresponding `genes`' `alleles`, normalized by the number of genes. This is ideal for:

- Continuous genetic representations
- Problems where the magnitude of differences matters
- Cases where you want to measure diversity based on numerical distances

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	diversity = rd.EuclideanDistance()
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let diversity = EuclideanDistance::new();
	```

---

## Species Management

Radiate implements species management to maintain population diversity through several mechanisms:

### Species Threshold

**All `diversity` output is normalized to a range of `0.0` to `1.0`, where `0.0` means no diversity and `1.0` means maximum diversity.
This means any threshold outside of these bounds will not work correctly**

The species threshold determines how similar individuals need to be to be considered part of the same `species`. A lower threshold will result in more species being formed, while a higher threshold will group more individuals into fewer species. This is crucial for controlling the balance between exploration and exploitation in the population. All of this is controlled by the `species_threshold` parameter in the engine:

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        diversity=diversity,
        species_threshold=.5  # Default value
    )
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

    let engine = GeneticEngine::builder()
        // ... other parameters ...
        .diversity(your_diversity)
        .species_threshold(0.5) // Default value
        // ... other parameters ...
        .build();
	```

A higher threshold means:

- More individuals will be considered part of the same species resulting in a fewer number of species
- Less diversity in the `population`
- Faster convergence

A lower threshold means:

- Fewer individuals will be considered part of the same species
- More diversity in the `population`
- Slower convergence

### Species Age

The `ecosystem` tracks the age of `species` to prevent stagnation, if a `species` reaches the given age limit without improvement, it will be removed from the `population`. This is controlled by the `max_species_age` parameter:

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        diversity=diversity,
        species_threshold=.5  # Default value
        max_species_age=20  # Default value
    )
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

    let engine = GeneticEngine::builder()
        // ... other parameters ...
        .diversity(your_diversity)
        .species_threshold(0.5) // Default value
        .max_species_age(20) // Default value
        // ... other parameters ...
        .build();
	```

This helps by:

- Limiting how long a species can survive without improvement
- Preventing dominant species from taking over the population
- Encouraging exploration of new solutions

## Best Practices

### Choosing a Diversity Measure

1. **For Binary/Discrete Problems**:
    - Use Hamming Distance
    - Good for problems where exact matches matter
    - Example: Binary optimization, discrete scheduling

2. **For Continuous Problems**:
    - Use Euclidean Distance
    - Better for problems where magnitude of differences matters
    - Example: Parameter optimization, function approximation

### Setting Species Threshold

1. **Start Conservative**:
    - Begin with the default value (`0.5`)
    - Monitor population diversity
    - Adjust based on convergence behavior

2. **Adjust Based on Problem**:
    - For problems requiring high diversity: Use lower values (`0.05`-`0.2`)
    - For problems needing faster convergence: Use higher values (`0.5`-`1.0`)

### Age Limits

1. **Species Age**:
    - Default (`20`) works well for most problems
    - Increase for complex problems requiring more exploration
    - Decrease for problems where quick convergence is desired

## Common Pitfalls

1. **Premature Convergence**:
    - Problem: Population converges too quickly to suboptimal solutions
    - Solution: 
        - Lower the species threshold
        - Increase max_species_age
        - Use a more aggressive mutation rate

2. **Excessive Diversity**:
    - Problem: Population fails to converge
    - Solution:
        - Increase the species threshold
        - Decrease max_species_age
        - Adjust selection pressure

3. **Stagnation**:
    - Problem: Population stops improving
    - Solution:
        - Decrease max_phenotype_age
        - Increase mutation rate
        - Adjust species threshold


## Example

Lets add on to our example - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll keep the previous inputs the same as before, but now we add `diversity` to the `GeneticEngine`.

=== ":fontawesome-brands-python: Python"

    ```python
    from typing import List
    import radiate as rd

    # Define a fitness function that uses the decoded values
    def fitness_function(individual: List[float]) -> float:    
        # Calculate how well these parameters fit your data
        a = individual[0]
        b = individual[1]
        return calculate_error(a, b)  # Your error calculation here

    # Create a codec for two parameters (a and b)
    codec = rd.FloatCodec.vector(
        length=2,                   # We need two parameters: a and b
        value_range=(-1.0, 1.0),    # Start with values between -1 and 1
        bound_range=(-10.0, 10.0)   # Allow evolution to modify the values between -10 and 10
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

    # Define the diversity measure
    diversity = rd.HammingDistance()  # or rd.EuclideanDistance() for continuous problems

    # Create the evolution engine
    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_function,
        offspring_selector=offspring_selector,
        survivor_selector=survivor_selector,
		alters=alters,
        diversity=diversity,  # Add the diversity measure
        species_threshold=0.5,  # Default value
        max_species_age=20,  # Default value
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

