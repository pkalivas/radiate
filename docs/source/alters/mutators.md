# Mutators

Mutators introduce (usually small) random changes to individual genes or chromosomes, helping maintain diversity in the population and enabling exploration of the search space.

---

## Uniform

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
	mutator = rd.UniformMutator(rate=rd.Rate.fixed(0.1))
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mutator = UniformMutator::new(0.1);
	let mutator = UniformMutator::from(Rate::fixed(0.1));
    ```

---

## Gaussian

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

## Arithmetic

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

## Swap

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

## Scramble

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

## Invert

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

## Polynomial

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


## Jitter

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
