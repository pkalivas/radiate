
# Diversity

Diversity is an opt-in aspect of radiate's genetic algorithm. At its core, this operator helps maintain a healthy population and prevent premature convergence. By using this operator, the `GeneticEngine` will split the `population` up into `species` by measuring the genetic distance, or diversity, between individuals during the evolution process. Its important to note that adding a diversity operator will increase the computational cost of the algorithm, so it should be used judiciously based on the problem at hand.

---

## Overview

Diversity in Radiate is implemented through two main components:

1. **Diversity Measurement**: Methods to quantify how different individuals are from each other. This is typically done by measuring the genetic distance between individuals - meaning the actual `chromosomes` and `genes` that make up the individuals.
2. **Species Management**: Mechanisms to group similar individuals and maintain population diversity

---

## Species Management

Radiate implements species management to maintain population diversity through several mechanisms:

### Species Threshold

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

