# Distance 

Distance measurements are used to quantify the distance between individuals in a population. Radiate provides several built-in distance measures that can be used to maintain diversity within the genetic algorithm. 

## Hamming Distance

**Compatible with**: `FloatGene<F>`, `IntGene<I>`, `BitGene`, `CharGene`, `PermuationGene<A>`

Hamming distance is the most straightforward distance measure, defined as the number of positions at which the corresponding genes are different.

- Binary or discrete genetic representations
- Problems where exact matches are important
- Cases where you want to measure diversity based on exact gene differences

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	diversity = rd.HammingDistance()
	diversity = rd.Dist.hamming() # using the dsl syntax
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let diversity = HammingDistance::new();
	```

---

## Euclidean Distance

**Compatible with**: `FloatGene<F>`, `IntGene<I>`

We define the Euclidean Distance as follows:

$$
D(x, y) = \sqrt{\sum_{i=1}^{n} (x_i - y_i)^2}
$$

or, the square root of sum of squared differences between corresponding `genes`' `alleles`. This is ideal for:

- Continuous genetic representations
- Problems where the magnitude of differences matters
- Cases where you want to measure diversity based on numerical distances


=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	diversity = rd.EuclideanDistance()
	diversity = rd.Dist.euclidean() # using the dsl syntax
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let diversity = EuclideanDistance::new();
	```

---

## Cosine Distance

**Compatible with**: `FloatGene<F>`, `IntGene<I>`

We define the Cosine Distance as follows:

$$
D(x, y) = 1 - \frac{\sum_{i=1}^{n
} x_i y_i}{\sqrt{\sum_{i=1}^{n} x_i^2} \sqrt{\sum_{i=1}^{n} y_i^2}}
$$

The Cosine Distance measures diversity by calculating the cosine of the angle between two vectors of gene values, which is particularly useful for:

- High-dimensional spaces
- Problems where the direction of the vector matters more than its magnitude
- Cases where you want to measure diversity based on the orientation of the gene vectors

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

	diversity = rd.CosineDistance()
	diversity = rd.Dist.cosine() # using the dsl syntax
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	use radiate::*;

	let diversity = CosineDistance::new();
	```

---

## Neat Distance

**Compatible with**: `GraphNode<Op<f32>>`

The Neat Distance measures diversity by using the [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) (NeuroEvolution of Augmenting Topologies) distance metric, which considers both structural and weight differences between neural network representations. This is particularly useful for:

- Neural network evolution
- Problems where both topology and weight differences matter
- Cases where you want to measure diversity based on neural network structure and weights

=== ":fontawesome-brands-python: Python"

	```python
	import radiate as rd

    # Parameters are: c1, c2, c3 - coefficients for excess genes, disjoint genes,
    # and average weight differences respectively
	diversity = rd.NeatDistance(excess=0.1, disjoint=1.0, weight_diff=0.5)
	diversity = rd.Dist.neat(0.1, 1.0, 0.5) # using the dsl syntax
	```

=== ":fontawesome-brands-rust: Rust"

    !!! note "requires `gp` feature flag"

	```rust
	use radiate::*;

    // Parameters are: c1, c2, c3 - coefficients for excess genes, disjoint genes,
    // and average weight differences respectively
	let diversity = NeatDistance::new(0.1, 1.0, 0.5);
	```
