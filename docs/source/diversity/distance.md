# Distance 

Distance measurements are used to quantify the distance between individuals in a population. Radiate provides several built-in distance measures that can be used to maintain diversity within the genetic algorithm. 

## Hamming Distance

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

## Euclidean Distance

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

## Cosine Distance

**Compatible with**: `FloatGene`, `IntGene<I>`

The Cosine Distance measures diversity by calculating the cosine similarity between the vectors representing individuals. This is particularly useful for:

- High-dimensional spaces
- Problems where the direction of the vector matters more than its magnitude
- Cases where you want to measure diversity based on the orientation of the gene vectors

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	diversity = rd.CosineDistance()
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let diversity = CosineDistance::new();
	```

---

## Neat Distance

**Compatible with**: `GraphNode<Op<f32>>`

The Neat Distance measures diversity by using the NEAT (NeuroEvolution of Augmenting Topologies) distance metric, which considers both structural and weight differences between neural network representations. This is particularly useful for:

- Neural network evolution
- Problems where both topology and weight differences matter
- Cases where you want to measure diversity based on neural network structure and weights

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

    # Parameters are: c1, c2, c3 - coefficients for excess genes, disjoint genes,
    # and average weight differences respectively
	diversity = rd.NeatDistance(excess=0.1, disjoint=1.0, weight_diff=0.5)
	```

=== ":fontawesome-brands-rust: Rust"

    !!! note "requires `gp` feature flag"

	```rust
	use radiate::*;

    // Parameters are: c1, c2, c3 - coefficients for excess genes, disjoint genes,
    // and average weight differences respectively
	let diversity = NeatDistance::new(0.1, 1.0, 0.5);
	```
