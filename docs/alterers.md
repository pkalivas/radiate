# Alterers

Alterers are the operators that modify the population of individuals through either mutation or crossover. Crossover and mutation are two powerful ways to create new individuals from the existing population, and they are essential for the genetic algorithm to explore the search space effectively.
As such, the choice of alterer can have a significant impact on the performance of the genetic algorithm, so it is important to choose an alterer that is well-suited to the problem being solved.

## Crossover 

Crossover is a genetic operator that combines two parent individuals to create one or more offspring. Radiate provides a number of built-in crossover operators that can be used to customize the crossover process, or you can define your own custom crossover operators.

Each crossover provides a `rate` parameter that determines the probability that the crossover will be applied to a pair of individuals. If the rate is set to `1.0`, the crossover will always be applied, while a rate of `0.0` will never apply the crossover. 

### MultiPoint

The `MultiPointCrossover` is a crossover operator that combines two parent individuals by selecting multiple crossover points and swapping the genetic material between the parents at those points. This is a 
classic crossover operator.

Create a new `MultiPointCrossover` with a crossover rate of `0.7` and 3 crossover points
```rust
let crossover = MultiPointCrossover::new(0.7, 3);
```

### Uniform

The `UniformCrossover` is a crossover operator creates new individuals by selecting genes from the parents with equal probability and swapping them between the parents. This is a simple crossover operator that can be effective in a wide range of problems.

Create a new `UniformCrossover` with a crossover rate of `0.7`
```rust
let crossover = UniformCrossover::new(0.7);
```

### Partially Mapped (PMX)

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

Create a new `PMXCrossover` with a crossover rate of `0.7`
```rust
let crossover = PMXCrossover::new(0.7);
```

### Shuffle

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

Create a new `ShuffleCrossover` with a crossover rate of `0.7`
```rust
let crossover = ShuffleCrossover::new(0.7);
```

### Mean

The `MeanCrossover` operator is a crossover mechanism designed 
for `NumericGene`s. It combines the corresponding genes of two parent chromosomes by 
replacing a gene in one chromosome with the mean (average) of the two genes. This approach 
is useful when genes represent numeric values such as weights or coordinates, 
as it promotes a balanced combination of parent traits.

Create a new `MeanCrossover` with a crossover rate of `0.7`
```rust
let crossover = MeanCrossover::new(0.7);
```

### Intermediate

The `IntermediateCrossover` operator is a crossover mechanism designed for `NumericGene`s. 
It combines the corresponding genes of two parent chromosomes by replacing a gene in one chromosome
with a value that lies between the two parent genes. The new gene is calculated as the weighted average
of the two parent genes, where the weight is determined by the `alpha` parameter.

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

Create a new `IntermediateCrossover` with a crossover rate of `0.7` and an alpha value of `0.5`
```rust
let crossover = IntermediateCrossover::new(0.7, 0.5);
```

## Mutation

Mutation is in charge of introducing genetic diversity into the population by randomly altering the genes of individuals. This is essential for exploring the search space of the problem. Each
mutator has a `rate` parameter that determines the probability that the mutation will be applied to a `Gene` - **Not** an individual. If the rate is set to `1.0`, the mutation will always be applied, while a rate of `0.0` will never apply the mutation.

!!! note

	The mutation rate is typically set to a low value, such as `0.1` or `0.01`. If the rate is too high,
	the algorithm is essentially performing random search, which is not efficient nor likely to find a good solution. If the rate is too low, the algorithm may get stuck in local optima.

### Gaussian

> Inputs
>
>  * `rate`: f32 - Mutation rate.
> * `std_dev`: f32 - The standard deviation of the Gaussian distribution.

The `GaussianMutator` operator is a mutation mechanism designed for `NumericGene`s. It introduces random noise to the gene values by adding a sample from a Gaussian distribution with a specified standard deviation. This mutation operator produces small, incremental changes centered around the current gene value.


Create a new `GaussianMutator` with a mutation rate of `0.1` and a standard deviation of `0.1`
```rust
let mutator = GaussianMutator::new(0.1, 0.1);
```


### Uniform

> Inputs
>
> * `rate`: f32 - Mutation rate.

`UniformMutator` is the most basic mutation operator. It randomly replaces a gene with a new instance of the gene type.

Create a new `UniformMutator` with a mutation rate of `0.1`
```rust
let mutator = UniformMutator::new(0.1);
```

### Arithmetic

> Inputs
>
> * `rate`: f32 - Mutation rate.

The `ArithmeticMutator` introduces diversity into genetic algorithms by mutating numerically based genes through basic arithmetic operations. It is designed to work on genes that support addition, subtraction, multiplication, and division.

1. Choose a random arithmetic operation: addition, subtraction, multiplication, or division.
2. Apply the operation to the gene value using a randomly generated value of the same gene type.
3. Replace the original gene with the result of the operation.

Create a new `ArithmeticMutator` with a mutation rate of `0.1`
```rust
let mutator = ArithmeticMutator::new(0.1);
```

### Invert

> Inputs
>
> * `rate`: f32 - Mutation rate.

`InvertMutator` is a segment inversion mutator. It randomly selects a segment of the chromosome and inverts the order of the genes within that segment. This mutation operator can be useful for problems where the order of `Gene`s matters, such as the Traveling Salesman Problem (TSP).

Create a new `InvertMutator` with a mutation rate of `0.1`
```rust
let mutator = InvertMutator::new(0.1);
```

### Swap

> Inputs
>
> * `rate`: f32 - Mutation rate.

The `SwapMutator` is a mutation operator designed for genetic algorithms to shuffle the positions of `Gene`s in a `Chromosome`. This mutator swaps two `Gene`s at randomly selected indices, introducing variability while maintaining the chromosome’s structural integrity. It is particularly suited for permutation-based problems.

Create a new `SwapMutator` with a mutation rate of `0.1`
```rust
let mutator = SwapMutator::new(0.1);
```
