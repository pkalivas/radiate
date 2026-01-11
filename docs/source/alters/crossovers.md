# Crossovers

Crossovers combine genetic material from two parents to create offspring, allowing good traits to be combined and propagated through the population.

---

## Blend

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

## Intermediate

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

## Mean

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

## Multi-Point

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

## Partially Mapped (PMX)

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

## Edge Recombination

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

## Shuffle

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

## Simulated Binary

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

## Uniform

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
