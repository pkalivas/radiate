# Distance

A **distance measure** is what speciation uses to decide whether two individuals are "similar." It takes two individuals and returns a single number, the smaller the number, the more alike two individuals are. The engine then compares this number against the [`species_threshold`](species.md#species-threshold) to group the population. Choosing a measure (and scaling the threshold to it) is the main lever you have over *how* the engine perceives diversity.

Below we see the built-in measurements `radiate` provides.

!!! note "Pick a measure that fits your genes"

	Each measure is defined only for certain gene types — Hamming works on any gene, while Euclidean and Cosine need numeric (`Float`/`Int`) genes. The **Compatible with** line on each measure tells you which.

| Measure | Produces | Value range | Reach for it when… |
|---|---|---|---|
| **Hamming** | fraction of genes that differ | `[0, 1]` | genes are discrete and *exact matches* matter |
| **Euclidean** | root-mean-squared difference | `0` → unbounded (scales with allele range) | numeric genes where the *magnitude* of difference matters |
| **Cosine** | `1 − cosine similarity` | `[0, 2]` (`[0, 1]` for non-negative values) | high-dimensional numeric genes where *direction* matters more than magnitude |
| **NEAT** | weighted structural + weight distance | `0` → unbounded | evolving graph/neural-network topologies |

!!! tip "Threshold follows the measure"

	Because each measure lives on a different scale, a `species_threshold` that works for Hamming (`[0, 1]`) is meaningless for an unbounded Euclidean distance. Always set the threshold relative to the range above — see [tuning the threshold](species.md#species-threshold).

---

## Hamming Distance

**Compatible with**: `FloatGene<F>`, `IntGene<I>`, `BitGene`, `CharGene`, `PermutationGene<A>`

This is really the simplest distance function. It's the percent of gene positions at which two individuals differ, normalized by the total number of genes so the result always falls in `[0, 1]` (`0` = identical, `1` = every gene differs). It only asks "are these two alleles equal?", which is why it works for any gene type.

Best for:

- Binary or discrete representations
- Problems where exact matches matter
- Measuring diversity by exact gene differences rather than magnitude

=== ":fontawesome-brands-python: Python"

	```python
	--8<-- "python/diversity/distance.py:hamming"
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	--8<-- "rust/diversity/distance.rs:hamming"
	```

---

## Euclidean Distance

**Compatible with**: `FloatGene<F>`, `IntGene<I>`

The root-mean-squared difference between corresponding alleles:

$$
D(x, y) = \sqrt{\frac{1}{n}\sum_{i=1}^{n} (x_i - y_i)^2}
$$

Dividing by the gene count `n` keeps the value comparable across genotypes of different lengths. The result is `0` for identical individuals and grows with the size of the differences, so its upper bound scales with your alleles' value range rather than being fixed.

Best for:

- Continuous (numeric) representations
- Problems where the *magnitude* of differences matters
- Measuring diversity by numerical distance

=== ":fontawesome-brands-python: Python"

	```python
	--8<-- "python/diversity/distance.py:euclidean"
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	--8<-- "rust/diversity/distance.rs:euclidean"
	```

---

## Cosine Distance

**Compatible with**: `FloatGene<F>`, `IntGene<I>`

One minus the cosine of the angle between the two genotypes treated as vectors:

$$
D(x, y) = 1 - \frac{\sum_{i=1}^{n} x_i y_i}{\sqrt{\sum_{i=1}^{n} x_i^2}\,\sqrt{\sum_{i=1}^{n} y_i^2}}
$$

This compares *orientation* rather than magnitude: two individuals pointing the same direction are close (`0`) even if one is scaled up. The value ranges over `[0, 2]` in general — and `[0, 1]` when all gene values are non-negative. A genotype whose values are all zero has no direction, so its distance to anything is defined as `1.0`.

Best for:

- High-dimensional spaces
- Problems where the *direction* of the gene vector matters more than its magnitude
- Measuring diversity by orientation rather than scale

=== ":fontawesome-brands-python: Python"

	```python
	--8<-- "python/diversity/distance.py:cosine"
	```

=== ":fontawesome-brands-rust: Rust"

	```rust
	--8<-- "rust/diversity/distance.rs:cosine"
	```

---

## NEAT Distance

**Compatible with**: `GraphNode<Op<f32>>`

The NEAT [compatibility distance](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) for evolving graph topologies. It aligns two graphs by their genes' historical *innovation* markers and combines three quantities, following Stanley's formula:

$$
D = c_1\frac{E}{N} + c_2\frac{D}{N} + c_3\,\overline{W}
$$

- **`E` — excess** genes (innovations beyond the other parent's newest gene), weighted by `c₁`
- **`D` — disjoint** genes (misaligned innovations within the shared range), weighted by `c₂`
- **`W̄` — average weight difference** across matching genes, weighted by `c₃`
- **`N`** normalizes the structural terms by the larger genome's size

This lets the engine separate genuinely different architectures from ones that merely differ in their connection weights.

=== ":fontawesome-brands-python: Python"

	```python
	--8<-- "python/diversity/distance.py:neat"
	```

=== ":fontawesome-brands-rust: Rust"

	!!! note "requires the `gp` feature flag"

		```rust
		--8<-- "rust/diversity/distance.rs:neat_distance"
		```
