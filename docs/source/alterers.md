
# Alterers

Alterers are genetic operators that modify the genetic material of individuals in a `population`. In Radiate, there are two main types of alterers:

1. **Mutators**: Operators that modify individual genes or chromosomes
2. **Crossovers**: Operators that combine genetic material from two parents to create offspring

These operators modify the `population` and are essential for the genetic algorithm to explore the search space effectively. As such, the choice of `alterer` can have a significant impact on the performance of the genetic algorithm, so it is important to choose an `alterer` that is well-suited to the problem being solved.


## Mutators

Mutators introduce (usually small) random changes to individual genes or chromosomes, helping maintain diversity in the population and enabling exploration of the search space.

---

### Uniform

> Inputs
> 
>   * `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**: Randomly changes genes to new random values
- **Best for**: General-purpose mutation when you want simple random changes
- **Example**: Binary or discrete genes where you want to flip values randomly
- **Compatible with**: `BitGene`, `CharGene`, `FloatGene`, `IntGene<I>`, `PermutationGene<A>`

The most basic mutation operator. It randomly replaces a gene with a new instance of the gene type.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    mutator = rd.UniformMutator(rate=0.1)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mutator = UniformMutator::new(0.1);
    ```

---

### Gaussian

> Inputs
> 
>   * `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**: Adds Gaussian (normal) noise to gene values
- **Best for**: Continuous values where you want small, normally distributed changes
- **Example**: Perfect for fine-tuning real-valued parameters in optimization problems
- **Compatible with**: `FloatGene`

The `GaussianMutator` operator is a mutation mechanism designed for `ArithmeticGene`s. It introduces random noise to the gene values by adding a sample from a Gaussian distribution with a specified standard deviation. This mutation operator produces small, incremental changes centered around the current gene value.
  
=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    mutator = rd.GaussianMutator(rate=0.1)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mutator = GaussianMutator::new(0.1);
    ```

---

### Arithmetic

> Inputs
> 
>   * `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**: Performs arithmetic operations (add, subtract, multiply, divide) on `genes`
- **Best for**: Genes that support arithmetic operations
- **Example**: Useful for numerical optimization where you want to explore the space through arithmetic operations
- **Compatible with**: `FloatGene`, `IntGene<I>`

The `ArithmeticMutator` introduces diversity into genetic algorithms by mutating numerically based `genes` through basic arithmetic operations. It is designed to work on `genes` that support addition, subtraction, multiplication, and division. Once the values have gone through their arithmatic operation, the result is clamped by the `gene`'s bounds to ensure it remains valid.

1. Choose a random arithmetic operation: addition, subtraction, multiplication, or division.
2. Apply the operation to the `gene` value using a randomly generated value of the same `gene` type.
3. Replace the original `gene` with the result of the operation.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    mutator = rd.ArithmeticMutator(rate=0.1)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mutator = ArithmeticMutator::new(0.1);
    ```

---

### Swap

> Inputs
>
> 	* `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**: Swaps positions of two `genes` within a `chromosome`
- **Best for**: Permutation problems (like TSP) or ordered sequences
- **Example**: Ideal for problems where gene order matters, like scheduling or routing
- **Compatible with**: `BitGene`, `CharGene`, `FloatGene`, `IntGene<I>`, `PermutationGene<A>`

The `SwapMutator` is a mutation operator designed for genetic algorithms to swap the positions of two `Gene`s in a `Chromosome`. This mutator swaps two `Gene`s at randomly selected indices, introducing variability while maintaining the `chromosome`s structural integrity. It is particularly suited for permutation-based problems.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    mutator = rd.SwapMutator(rate=0.1)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mutator = SwapMutator::new(0.1);
    ```

---

### Scramble

> Inputs
>
> 	* `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**: Randomly reorders a segment of `genes`
- **Best for**: Breaking up local optima in ordered sequences
- **Example**: Useful for permutation problems where you want to explore different orderings
- **Compatible with**: `BitGene`, `CharGene`, `FloatGene`, `IntGene<I>`, `PermutationGene<A>`

The `ScrambleMutator` randomly reorders a segment of `genes` within a `chromosome`. 

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	mutator = rd.ScrambleMutator(rate=0.1)
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let mutator = ScrambleMutator::new(0.1);
	```

---

### Invert

> Inputs
>
> 	* `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**: Reverses the order of a segment of `genes`
- **Best for**: Ordered sequences where reverse ordering might be beneficial
- **Example**: Helpful in permutation problems where reverse ordering of segments might lead to better solutions
- **Compatible with**: `BitGene`, `CharGene`, `FloatGene`, `IntGene<I>`, `PermutationGene<A>`

`InvertMutator` is a segment inversion mutator. It randomly selects a segment of the `chromosome` and inverts the order of the `genes` within that segment.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	mutator = rd.InvertMutator(rate=0.1)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let mutator = InvertMutator::new(0.1);
	```

### Polynomial

> Inputs
>
> 	* `rate`: f32 - Mutation rate (0.0 to 1.0)
> 	* `eta`: f32 - Exponent for polynomial mutation

- **Purpose**: Applies a polynomial mutation to the genes
- **Best for**: Continuous optimization problems
- **Example**: Multi-objective optimization, bounded domains where classic Gaussian mutation may overshoot
- **Compatible with**: `FloatGene`

The `PolynomialMutator` applies a polynomial mutation to the genes of a chromosome. This provides a bounded and unbiased mutation to genes where you care about the distribution of the mutation. Unlike Gaussian mutation, Polynomial can give more control over the tail behavior.

The `eta` parameter controls the shape of the mutation distribution. A higher `eta` value results in a more exploratory mutation, while a lower value makes the mutation more exploitative. For example, a low `eta` (1.0-5.0) leads to bigger mutations, while a high value (20.0-100.0) leads to smaller, more fine grained mutations.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	mutator = rd.PolynomialMutator(rate=0.1, eta=20.0)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let mutator = PolynomialMutator::new(0.1, 20.0);
	```


### Jitter

> Inputs
> 
> * `rate`: f32 - Mutation rate (0.0 to 1.0)
> * `magnitude`: f32 - Maximum jitter magnitude

- **Purpose**: Adds small random perturbations to gene values
- **Best for**: Fine-tuning solutions in continuous spaces
- **Example**: Useful for exploring the neighborhood of a solution
- **Compatible with**: `FloatGene`

The `JitterMutator` adds small random perturbations to the values of a `gene` within a `chromosome`. A random value is sampled from a uniform distribution between [-1, 1], then it is scaled by the `magnitude` parameter and added to the current gene value. This mutation operator is particularly useful for fine-tuning solutions in continuous spaces, as it allows for small adjustments that can help explore the local neighborhood of a solution.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	mutator = rd.JitterMutator(rate=0.1, magnitude=0.5)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let mutator = JitterMutator::new(0.1, 0.5);
	```

---

## Crossovers

Crossovers combine genetic material from two parents to create offspring, allowing good traits to be combined and propagated through the population.

---

### Blend

> Inputs
>
> 	* `rate`: f32 - Crossover rate (0.0 to 1.0)
> 	* `alpha`: f32 - Blending factor (0.0 to 1.0)

- **Purpose**: Creates offspring by blending parent genes using a weighted average
- **Best for**: Continuous optimization problems
- **Example**: Useful when you want to explore the space between parent solutions
- **Compatible with**: `FloatGene`, `IntGene<I>`

The `BlendCrossover` is a crossover operator designed for `ArithmeticGene`s. It introduces variability by blending the `gene` controlled by the `alpha` parameter. This approach allows for smooth transitions between `gene` values, promoting exploration of the search space.
Its functionality is similar to the `IntermediateCrossover`, but it uses a different formula to calculate the new `gene` value.
Its defined as:

$$
\text{allele}_{\text{child}} = \text{allele}_{\text{parent1}} - \alpha \cdot (\text{allele}_{\text{parent2}} - \text{allele}_{\text{parent1}})
$$

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	crossover = rd.BlendCrossover(rate=0.1, alpha=0.5)
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = BlendCrossover::new(0.1, 0.5);
	```
---

### Intermediate

> Inputs
>
> 	* `rate`: f32 - Crossover rate (0.0 to 1.0)
> 	* `alpha`: f32 - Blending factor (0.0 to 1.0)

- **Purpose**: Similar to blend crossover but uses a different blending formula
- **Best for**: Real-valued optimization problems
- **Example**: Good for fine-tuning solutions in continuous spaces
- **Compatible with**: `FloatGene`

The `IntermediateCrossover` operator is a crossover mechanism designed for `ArithmeticGene`s. 
It combines the corresponding `genes` of two parent chromosomes by replacing a gene in one chromosome
with a value that lies between the two parent `genes`. The new gene is calculated as the weighted average
of the two parent `genes`, where the weight is determined by the `alpha` parameter.

1. Input:
	* Two parent chromosomes (Parent 1 and Parent 2) composed of real-valued genes.
	* A crossover `rate`, determining the probability of applying the operation for each gene.
	* An interpolation parameter (`alpha`), which controls the weight given to each parent’s gene during crossover.
2. Weighted Interpolation:
	* For each gene position in the parents:
	* Generate a random value between 0 and 1.
	* If the random value is less than the rate, compute a new allele as a weighted combination of the parent’s alleles:

	$$
	\text{allele}_{\text{child}} = \alpha \cdot \text{allele}_{\text{parent1}} + (1 - \alpha) \cdot \text{allele}_{\text{parent2}}
	$$ 
	
	* Here, ${alpha}$ is randomly sampled from the range [0, ${self.alpha}$].

3. Modify Genes:
	* Replace the gene in Parent 1 with the newly calculated gene value. Parent 2 remains unmodified.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd
	
	crossover = rd.IntermediateCrossover(rate=0.1, alpha=0.5)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = IntermediateCrossover::new(0.1, 0.5);
	```

---

### Mean

> Inputs
>
> 	* `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Creates offspring by taking the mean of parent genes
- **Best for**: Problems where averaging parent values is meaningful
- **Example**: Useful for numerical optimization where the average of good solutions might be better
- **Compatible with**: `FloatGene`, `IntGene<I>`

The `MeanCrossover` operator is a crossover mechanism designed 
for `ArithmeticGene`s. It combines the corresponding `genes` of two parent chromosomes by 
replacing a gene in one chromosome with the mean (average) of the two `genes`. This approach 
is useful when `genes` represent numeric values such as weights or coordinates, 
as it promotes a balanced combination of parent traits.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	crossover = rd.MeanCrossover(rate=0.1)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = MeanCrossover::new(0.1);
	```

---

### Multi-Point

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)
>  * `num_points`: int - Number of crossover points (typically 1 or 2)

- **Purpose**: Swaps segments between parents at multiple points
- **Best for**: General-purpose crossover for most problems
- **Example**: Classic genetic algorithm crossover, good for most applications
- **Compatible with**: `FloatGene`, `IntGene<I>`, `BitGene`, `CharGene`, `PermutationGene<A>`

The `MultiPointCrossover` is a crossover operator that combines two parent individuals by selecting multiple crossover points and swapping the genetic material between the parents at those points. This is a 
classic crossover operator.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	crossover = rd.MultiPointCrossover(rate=0.1, num_points=2)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = MultiPointCrossover::new(0.1, 2);
	```

---

### Partially Mapped (PMX)

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Specialized crossover for permutation problems
- **Best for**: Permutation problems (TSP, scheduling)
- **Example**: Ideal for problems where gene order matters and no duplicates are allowed
- **Compatible with**: `PermutationGene<A>`

The `PMXCrossover` is a genetic algorithm crossover technique used for problems where solutions are represented as permutations.
It is widely used in combinatorial optimization problems, such as the Traveling Salesman Problem (TSP),
where the order of elements in a solution is significant.

1. Two random crossover points are selected, dividing the parents into three segments: left, middle, and right.
	* The middle segment defines the “mapping region.”
3.	Mapping Region:
	* The elements between the crossover points in Parent 1 and Parent 2 are exchanged to create mappings.
	* These mappings define how elements in the offspring are reordered.
4.	Child Construction:
	* The middle segment of one parent is directly copied into the child.
	* For the remaining positions, the mapping ensures that no duplicate elements are introduced:
	* If an element is already in the middle segment, its mapped counterpart is used.
	* This process continues recursively until all positions are filled.

=== ":fontawesome-brands-python: Python"
	```python
	import radiate as rd

	crossover = rd.PartiallyMappedCrossover(rate=0.1)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = PMXCrossover::new(0.1);
	```

### Edge Recombination

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Combines edges from both parents
- **Best for**: Permutation problems (TSP, scheduling)
- **Example**: Useful for problems where the relationship between genes is important
- **Compatible with**: `PermutationGene<A>`

The `EdgeRecombinationCrossover` is a specialized crossover operator for permutation problems. It focuses on preserving the connectivity between genes by combining edges from both parents.

1. **Edge List Creation**:
	* For each parent, create a list of edges representing the connections between genes.
2. **Edge Selection**:
	* Randomly select edges from both parents to create a new offspring.
	* This selection process ensures that the offspring inherits important connections from both parents.
3. **Child Construction**:
	* The selected edges are used to construct the offspring's gene sequence.
	* This process helps maintain the overall structure and relationships present in the parent solutions.
  
**Example**: If Parent 1 has edges (A-B, B-C) and Parent 2 has edges (B-C, C-D), the offspring might inherit edges (A-B, C-D).

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	crossover = rd.EdgeRecombinationCrossover(rate=0.1)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = EdgeRecombinationCrossover::new(0.1);
	```

---

### Shuffle

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Shuffles genes before performing crossover
- **Best for**: Problems where gene position is important
- **Example**: Useful for problems where you want to maintain gene independence while exploring new combinations
- **Compatible with**: `FloatGene`, `IntGene<I>`, `BitGene`, `CharGene`, `PermutationGene<A>`

The `ShuffleCrossover` is a crossover operator used in genetic algorithms, 
particularly when working with permutations or chromosomes where order matters. 
It works by shuffling the order in which genes are exchanged between two parent chromosomes 
to introduce randomness while preserving valid gene configurations.

1. Determine Gene Indices:
	* Generate a list of indices corresponding to the positions in the chromosomes.
	* Shuffle these indices to randomize the order in which genes will be swapped.
2.	Swap Genes Alternately:
	* Iterate over the shuffled indices.
	* For even indices, copy the gene from Parent 2 into the corresponding position in Child 1, and vice versa for odd indices.
3.	Result:
	* Two offspring chromosomes are produced with genes shuffled and swapped in random positions.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	crossover = rd.ShuffleCrossover(rate=0.1)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = ShuffleCrossover::new(0.1);
	```

---

### Simulated Binary

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)
>  * `contiguity`: f32 - Contiguity factor (0.0 to 1.0)

- **Purpose**: Simulates binary crossover for real-valued genes
- **Best for**: Real-valued optimization problems
- **Example**: Good for problems where you want to maintain diversity while exploring the space
- **Compatible with**: `FloatGene`

The `SimulatedBinaryCrossover` is a crossover operator designed for `FloatGene`s. It simulates binary crossover by creating offspring that are a linear combination of the parents, controlled by a contiguity factor. Effectively, it allows for a smooth transition between parent values while maintaining the overall structure of the `genes` by smampling from a uniform distribution between the two parents.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	crossover = rd.SimulatedBinaryCrossover(rate=0.1, contiguity=0.5)
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = SimulatedBinaryCrossover::new(0.1, 0.5);
	```

---

### Uniform

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Randomly selects genes from either parent
- **Best for**: Problems where gene independence is high
- **Example**: Useful when genes have little interaction with each other
- **Compatible with**: `FloatGene`, `IntGene<I>`, `BitGene`, `CharGene`, `PermutationGene<A>`

The `UniformCrossover` is a crossover operator creates new individuals by selecting `genes` from the parents with equal probability and swapping them between the parents. This is a simple crossover operator that can be effective in a wide range of problems.

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	crossover = rd.UniformCrossover(rate=0.1)
	```
=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let crossover = UniformCrossover::new(0.1);
	```

---


## Best Practices

1. **Rate Selection**:
    - Start with conservative rates (0.01 for mutation, 0.5-0.8 for crossover)
    - Adjust based on problem characteristics
    - Higher rates increase exploration but may disrupt good solutions

2. **Choosing the Right Alterer**:
    - For continuous problems: Use Gaussian or Arithmetic mutators with Blend/Intermediate crossover
    - For permutation problems: Use Swap/Scramble mutators with PMX or Shuffle crossover
    - For binary problems: Use Uniform mutator with Multi-point or Uniform crossover

3. **Combining Alterers**:
    - It's often beneficial to use multiple alterers
    - Example: Combine a local search mutator (Gaussian) with a global search crossover (Multi-point)
    - Monitor population diversity to ensure proper balance

4. **Parameter Tuning**:
    - Start with default parameters
    - Adjust based on problem size and complexity
    - Use smaller rates for larger problems

## Common Pitfalls

1. **Too High Mutation Rates**:
    - Can lead to random search behavior
    - May destroy good solutions before they can be exploited
    - Solution: Start with low rates (0.01-0.1) and adjust based on results

2. **Inappropriate Crossover Selection**:
    - Using permutation crossovers for continuous problems
    - Using continuous crossovers for permutation problems
    - Solution: Match the crossover type to your problem domain

3. **Ignoring Problem Constraints**:
    - Some alterers may produce invalid solutions
    - Solution: Use appropriate alterers or implement repair mechanisms

4. **Poor Parameter Tuning**:
    - Using the same parameters for all problems
    - Solution: Experiment with different parameters and monitor performance

---

## Example

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

	// There are a few different ways we can add alters to the engine in rust. Assumming you 
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
