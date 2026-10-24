# Crossovers

Crossovers combine genetic material from two parents to create offspring, allowing good traits to be combined and propagated through the population.

| Crossover | Works on | What it does |
|---|---|---|
| [Blend](#blend) | Float | child = parent ± α·(difference); can reach past a parent |
| [Differential](#differential) | Float | differential evolution: moves genes toward `a + F·(b − c)` from three donors |
| [Intermediate](#intermediate) | Float | weighted average of the two parents' alleles |
| [Mean](#mean) | Float, Int | child gene = mean of the two parents' genes |
| [Multi-Point](#multi-point) | Bit, Char, Float, Int | swaps segments at N cut points (classic GA crossover) |
| [Partially Mapped (PMX)](#partially-mapped-pmx) | Permutation | preserves absolute positions from both parents |
| [Order (OX1)](#order-ox1) | Permutation | preserves relative order from both parents |
| [Edge Recombination](#edge-recombination) | Permutation | preserves adjacency/edges from both parents |
| [Shuffle](#shuffle) | Bit, Char, Float, Int | shuffles positions, then swaps genes |
| [Simulated Binary](#simulated-binary) | Float | real-valued analog of binary crossover (contiguity-controlled) |
| [Uniform](#uniform) | Bit, Char, Float, Int | each gene independently taken from either parent |

---

## Blend

> Inputs
>
> 	* `rate`: f32 - Crossover rate (0.0 to 1.0)
> 	* `alpha`: f32 - Blending factor (0.0 to 1.0)

- **Purpose**: Creates offspring by extrapolating each parent's gene away from the other parent, scaled by `alpha`
- **Best for**: Continuous optimization problems
- **Example**: Useful when you want to explore just beyond the region spanned by the parent solutions
- **Compatible with**: `FloatGene`

The `BlendCrossover` is a crossover operator designed for `FloatGene`s. It introduces variability by moving each `gene` away from the other parent's value by a fraction `alpha` of their difference, so children land outside the interval between the two parents rather than inside it. Contrast this with the `IntermediateCrossover`, which interpolates *between* the parents.
Its defined as:

$$
\text{allele}_{\text{child}} = \text{allele}_{\text{parent1}} - \alpha \cdot (\text{allele}_{\text{parent2}} - \text{allele}_{\text{parent1}})
$$

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:blend_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:blend_crossover"
    ```
---

## Differential

> Inputs
>
>  * `rate`: f32 - Chance each individual becomes a target (0.0 to 1.0)
>  * `weight`: f32 - How far along the difference vector the donor moves, often called `F`. Must be in (0, 2]
>  * `crossover_rate`: f32 - Chance each gene takes the donor's value, often called `CR` (0.0 to 1.0)

- **Purpose**: Differential evolution (DE/rand/1/bin) as a crossover
- **Best for**: Continuous optimization, especially with interacting variables
- **Compatible with**: `FloatGene`

For each target, three other distinct individuals `a`, `b` and `c` are drawn from the population. Each gene of the target is replaced, with probability `crossover_rate`, by

$$
v = a + F \cdot (b - c)
$$

and one randomly chosen gene always takes `v`, so every target changes. Values are clamped to each gene's bounds. All donors are read from the population as it was before the step.

Textbook DE only keeps a trial if it beats its target. An alterer can't evaluate fitness, so here the trial always replaces the target and the engine's survivor selection does the filtering. The population needs at least 4 individuals.

!!! tip "Choosing `weight` and `crossover_rate`"

    A `weight` of `0.5` is the standard starting point; raise it toward `0.8` if the population converges too early. Use a high `crossover_rate` (around `0.9`) when the variables interact and a low one (around `0.1`) when they're independent.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:differential_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:differential_crossover"
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

The `IntermediateCrossover` operator is a crossover mechanism designed for `FloatGene`s. 
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
    
    * Here, the blend factor $\alpha$ is randomly sampled from the range $[0, \text{alpha}]$ for each gene, where `alpha` is the configured parameter.

3. Modify Genes:
    * Replace the gene in Parent 1 with the newly calculated gene value. Parent 2 remains unmodified.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:intermediate_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:intermediate_crossover"
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
for `NumericGene`s (`FloatGene` and `IntGene<I>`). It combines the corresponding `genes` of two parent chromosomes by 
replacing a gene in one chromosome with the mean (average) of the two `genes`. This approach 
is useful when `genes` represent numeric values such as weights or coordinates, 
as it promotes a balanced combination of parent traits.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:mean_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:mean_crossover"
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
- **Compatible with**: `FloatGene`, `IntGene<I>`, `BitGene`, `CharGene`
- **Not for**: `PermutationGene<A>` — swapping genes position-by-position between parents creates duplicates, producing invalid permutations. Use [PMX](#partially-mapped-pmx) or [Edge Recombination](#edge-recombination) instead.

The `MultiPointCrossover` is a crossover operator that combines two parent individuals by selecting multiple crossover points and swapping the genetic material between the parents at those points. This is a 
classic crossover operator.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:multipoint_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:multipoint_crossover"
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
2.	Mapping Region:
    * The elements between the crossover points in Parent 1 and Parent 2 are exchanged to create mappings.
    * These mappings define how elements in the offspring are reordered.
3.	Child Construction:
    * The middle segment of one parent is directly copied into the child.
    * For the remaining positions, the mapping ensures that no duplicate elements are introduced:
    * If an element is already in the middle segment, its mapped counterpart is used.
    * This process continues recursively until all positions are filled.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:pmx_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:pmx_crossover"
    ```

---

## Order (OX1)

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Combines two permutations while keeping the relative order of genes
- **Best for**: Scheduling and sequencing, where "a comes before b" matters more than absolute position
- **Compatible with**: `PermutationGene<A>`

The `OrderCrossover` (Davis, 1985) produces two children. Each child keeps a random segment of its own parent in place, then fills the remaining positions with the missing genes in the order they appear in the other parent. Filling starts just after the segment and wraps around.

Where PMX preserves *absolute positions* and edge recombination preserves *adjacency*, order crossover preserves *relative order*.

**Example**: parents `[1,2,3,4,5,6,7,8,9]` and `[4,5,2,1,8,7,6,9,3]` with the segment at positions 3 through 6:

```text
child one: [2,1,8 | 4,5,6,7 | 9,3]
child two: [3,4,5 | 1,8,7,6 | 9,2]
```

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:order_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:order_crossover"
    ```

---

## Edge Recombination

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Combines edges from both parents
- **Best for**: Permutation problems (TSP, scheduling)
- **Example**: Useful for problems where the relationship between genes is important
- **Compatible with**: `PermutationGene<A>`

The `EdgeRecombinationCrossover` is a specialized crossover operator for permutation problems. It builds a child that keeps as many of the parents' edges (adjacent pairs) as possible.

1. **Edge Table**:
    * For every gene, list its neighbors in both parents. Each gene has at most four.
2. **Walk**:
    * Start from a random gene. At each step, move to the current gene's unvisited neighbor that has the fewest unvisited neighbors left, breaking ties at random.
    * Picking the most constrained neighbor first keeps the walk from painting itself into a corner.
3. **Dead Ends**:
    * If the current gene has no unvisited neighbors, jump to a random unvisited gene.
  
Because it follows the parents' edges, the child usually inherits almost all of its adjacencies from one parent or the other, which is why it works well on routing problems like TSP. It produces one child per crossover.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:edge_recombination_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:edge_recombination_crossover"
    ```

---

## Shuffle

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Shuffles genes before performing crossover
- **Best for**: Problems where gene position is important
- **Example**: Useful for problems where you want to maintain gene independence while exploring new combinations
- **Compatible with**: `FloatGene`, `IntGene<I>`, `BitGene`, `CharGene`
- **Not for**: `PermutationGene<A>` — swapping genes position-by-position between parents creates duplicates, producing invalid permutations. Use [PMX](#partially-mapped-pmx) or [Edge Recombination](#edge-recombination) instead.

The `ShuffleCrossover` is a crossover operator used in genetic algorithms. 
It works by shuffling the order in which genes are exchanged between two parent chromosomes 
to introduce randomness. Genes are still swapped position-for-position, so it is not suitable for permutations.

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
    --8<-- "python/alters/crossovers.py:shuffle_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:shuffle_crossover"
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

The `SimulatedBinaryCrossover` is a crossover operator designed for `FloatGene`s. It simulates binary crossover by creating offspring that are a linear combination of the parents, controlled by a contiguity factor. Effectively, it allows for a smooth transition between parent values while maintaining the overall structure of the `genes` by sampling from a uniform distribution between the two parents.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:sbx_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:sbx_crossover"
    ```

---

## Uniform

> Inputs
>
>  * `rate`: f32 - Crossover rate (0.0 to 1.0)

- **Purpose**: Randomly selects genes from either parent
- **Best for**: Problems where gene independence is high
- **Example**: Useful when genes have little interaction with each other
- **Compatible with**: `FloatGene`, `IntGene<I>`, `BitGene`, `CharGene`
- **Not for**: `PermutationGene<A>` — swapping genes position-by-position between parents creates duplicates, producing invalid permutations. Use [PMX](#partially-mapped-pmx) or [Edge Recombination](#edge-recombination) instead.

The `UniformCrossover` is a crossover operator creates new individuals by selecting `genes` from the parents with equal probability and swapping them between the parents. This is a simple crossover operator that can be effective in a wide range of problems.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/crossovers.py:uniform_crossover"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/crossovers.rs:uniform_crossover"
    ```
