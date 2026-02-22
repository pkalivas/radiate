from radiate.operators.selector import (
    TournamentSelector,
    RouletteSelector,
    NSGA2Selector,
    NSGA3Selector,
    EliteSelector,
    BoltzmannSelector,
    RankSelector,
    LinearRankSelector,
    StochasticSamplingSelector,
    TournamentNSGA2Selector,
)
from radiate.operators.alterer import (
    BlendCrossover,
    IntermediateCrossover,
    ArithmeticMutator,
    SwapMutator,
    UniformCrossover,
    UniformMutator,
    MultiPointCrossover,
    MeanCrossover,
    ShuffleCrossover,
    SimulatedBinaryCrossover,
    PartiallyMappedCrossover,
    GaussianMutator,
    GraphCrossover,
    OperationMutator,
    GraphMutator,
    TreeCrossover,
    HoistMutator,
    InversionMutator,
    PolynomialMutator,
    EdgeRecombinationCrossover,
    JitterMutator,
    ScrambleMutator,
)
from radiate.operators.limit import (
    ScoreLimit,
    GenerationsLimit,
    SecondsLimit,
    ConvergenceLimit,
    MetricLimit,
)
from radiate.operators.distance import (
    EuclideanDistance,
    CosineDistance,
    NeatDistance,
    HammingDistance,
)

from radiate.operators.rate import Rate


class Select:
    @staticmethod
    def tournament(k: int = 3):
        """
        The `TournamentSelector` is a selection strategy that selects individuals from the `population` by
        holding a series of tournaments. In each tournament, a random subset of size `k` of individuals
        is selected, and the fittest individual from that subset is chosen. This can help to maintain
        diversity in the `population` and prevent premature convergence by allowing weaker individuals to be selected occasionally.

        :param k: Tournament size.
        """
        return TournamentSelector(k=k)

    @staticmethod
    def roulette():
        """
        The `RouletteSelector` is a selection strategy that selects individuals from the `population`
        based on their fitness values. The probability of an individual being selected is proportional
        to its fitness value, so fitter individuals are more likely to be chosen. The probability of an
        individual being selected can be thought of as:

        P(i) = f(i) / Σ f(j) for all j in population

        Although the implementation itself is a bit more mathematically complex to ensure accuracy.
        This is an extremely popular selection strategy due to its simplicity and effectiveness.
        """
        return RouletteSelector()

    @staticmethod
    def nsga2():
        """
        The `NSGA2Selector` is a selection strategy used in multi-objective optimization problems.

        It is based on the Non-Dominated Sorting Genetic Algorithm II (NSGA-II) and selects
        individuals based on their Pareto dominance rank and crowding distance. The NSGA-II algorithm
        is designed to maintain a diverse set of solutions that represent the trade-offs between multiple conflicting objectives.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * Individuals in the same front are then ranked based on their crowding distance, which measures the density of solutions around them.
        * Individuals with lower ranks and higher crowding distances are more likely to be selected.
        """
        return NSGA2Selector()

    @staticmethod
    def nsga3(points: int = 12):
        """
        The `NSGA3Selector` is a selection strategy used in multi-objective optimization problems, based on the NSGA-III algorithm. It extends the NSGA-II algorithm by introducing reference points to guide the selection process towards a well-distributed set of solutions across the Pareto front.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * Reference points are generated in the objective space to represent desired trade-offs between objectives.
        * Individuals are selected based on their proximity to these reference points, ensuring a diverse set of solutions across the Pareto front.

        :param points: The number of reference points to use for guiding the selection process.
        """
        return NSGA3Selector(points=points)

    @staticmethod
    def elite():
        """
        The `EliteSelector` is a selection strategy that selects the top `n` individuals from the population
        based on their fitness values. This can be useful for preserving the best individuals
        in the population and preventing them from being lost during the selection process.
        """
        return EliteSelector()

    @staticmethod
    def boltzmann(temp: float = 1.0):
        """
        The `BoltzmannSelector` is a probabilistic selection strategy inspired by the Boltzmann distribution
        from statistical mechanics, where selection probabilities are scaled based on temperature.
        Temperature influences the balance between exploration and exploitation during the algorithm’s run.

        As the temperature decreases, the selection process becomes more deterministic,
        with fitter individuals being more likely to be selected. Conversely, as the temperature increases,
        the selection process becomes more random, with all individuals having an equal chance of being selected.

        :param temp: Temperature for the Boltzmann selector.
        """
        return BoltzmannSelector(temp=temp)

    @staticmethod
    def rank():
        """
        The `RankSelector` is a selection strategy that selects individuals from the `population`
        based on their rank, or index, in the `population`. The fitness values of the individuals are
        first ranked, and then the selection probabilities are assigned based on these ranks.
        This helps to maintain diversity in the population and prevent premature convergence by ensuring that
        all individuals have a chance to be selected, regardless of their fitness values.
        """
        return RankSelector()

    @staticmethod
    def linear_rank(pressure: float = 1.5):
        """
        The `LinearRankSelector` is a selection strategy that selects individuals from the
        `population` based on their rank, or index, in the `population`, but with a linear scaling
        of the selection probabilities. The fitness values of the individuals are first ranked, and
        then the scaling factor is applied to the ranks. This helps to maintain diversity
        in the `population` and prevent premature convergence by ensuring that all individuals have a
        chance to be selected, but with a bias towards fitter individuals. The linear scaling function can be thought of as:

        P(i) = (2 - pressure) / N + (2 * (rank(i) - 1) * (pressure - 1)) / (N * (N - 1))

        A higher `pressure` will result in a stronger bias towards fitter individuals, while a lower value will result in a more uniform selection.

        :param pressure: Pressure for the linear rank selector.
        """
        return LinearRankSelector(pressure=pressure)

    @staticmethod
    def stochastic_universal_sampling():
        """
        Stochastic Universal Sampling (SUS) is a probabilistic selection technique used to ensure that selection is
        proportional to fitness, while maintaining diversity. Some consider it an improvement over roulette wheel selection,
        designed to reduce bias and randomness in the selection process by ensuring all individuals have a chance to be chosen,
        proportional to their fitness values.

        1. Fitness Proportional Selection:
            * Each individual in the population is assigned a segment of a virtual “roulette wheel,” where the size of
              the segment is proportional to the individual's fitness.
            * Individuals with higher fitness occupy larger segments.
        * Single Spin with Multiple Pointers:
            * Unlike traditional roulette wheel selection, which spins the wheel multiple times (once per selection), SUS
              uses a single spin and places multiple evenly spaced pointers on the wheel.
            * The distance between the pointers is: `d = total_fitness / n`, where `n` is the number of individuals to select.
        * Selection:
            * The wheel is spun once, and the pointers are placed on the wheel at random positions.
            * Individuals whose segments are intersected by the pointers are selected.
        """
        return StochasticSamplingSelector()

    @staticmethod
    def tournament_nsga2(k: int = 3):
        """
        The `TournamentNSGA2Selector` is a selection strategy that combines the principles of tournament
        selection with the NSGA-II algorithm. It selects individuals based on their Pareto dominance
        rank and crowding distance, but uses a tournament-style approach to select individuals from each Pareto front.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * A tournament is held within each Pareto front, where a random subset of individuals is selected.
        * The winner of the tournament is selected based on their crowding distance, which measures the density of solutions around them.

        :param k: Tournament size.
        """
        return TournamentNSGA2Selector(k=k)


class Cross:
    @staticmethod
    def sbx(rate: Rate | float = 0.1, contiguity: float = 0.5):
        """
        The `SimulatedBinaryCrossover` is a crossover operator designed for `FloatGene`s.
        It simulates binary crossover by creating offspring that are a linear combination of the parents, controlled
        by a contiguity factor. Effectively, it allows for a smooth transition between parent values
        while maintaining the overall structure of the `genes` by sampling from a uniform distribution between the two parents.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param contiguity: The contiguity factor that influences the distribution of offspring `gene` values between the parents.
        """
        return SimulatedBinaryCrossover(rate, contiguity)

    @staticmethod
    def pmx(rate: Rate | float = 0.1):
        """
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

        :param rate: The probability of applying the crossover to a pair of parents.
        """
        return PartiallyMappedCrossover(rate)

    @staticmethod
    def multipoint(rate: Rate | float = 0.1, num_points: int = 2):
        """
        The `MultiPointCrossover` is a crossover operator that combines two parent individuals by selecting
        multiple crossover points and swapping the genetic material between the parents at those points. This is a
        classic crossover operator.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param num_points: The number of crossover points to use.
        """
        return MultiPointCrossover(rate, num_points)

    @staticmethod
    def mean(rate: Rate | float = 0.1):
        """
        The `MeanCrossover` operator is a crossover mechanism designed
        for `ArithmeticGene`s. It combines the corresponding `genes` of two parent chromosomes by
        replacing a gene in the first parent with the mean (average) of the two `genes`. This approach
        is useful when `genes` represent numeric values such as weights or coordinates,
        as it promotes a balanced combination of parent traits.

        :param rate: The probability of applying the crossover to a pair of parents.
        """
        return MeanCrossover(rate)

    @staticmethod
    def uniform(rate: Rate | float = 0.1):
        """
        The `UniformCrossover` is a crossover operator creates new individuals by selecting `genes`
        from the parents with equal probability and swapping them between the parents.
        This is a simple crossover operator that can be effective in a wide range of problems.

        :param rate: The probability of applying the crossover to a pair of parents.
        """
        return UniformCrossover(rate)

    @staticmethod
    def blend(rate: Rate | float = 0.1, alpha: float = 0.5):
        """
        The `BlendCrossover` is a crossover operator designed for `ArithmeticGene`s (IntGene & FloatGene).
        It introduces variability by blending the `gene` controlled by the `alpha` parameter.
        This approach allows for smooth transitions between `gene` values, promoting exploration of the search space.
        Its functionality is similar to the `IntermediateCrossover`, but it uses a different formula to calculate the new `gene` value.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param alpha: The blending factor that determines the extent of blending between parent `gene` values.
        """
        return BlendCrossover(rate, alpha)

    @staticmethod
    def intermediate(rate: Rate | float = 0.1, alpha: float = 0.5):
        """
        The `IntermediateCrossover` operator is a crossover mechanism designed for `ArithmeticGene`s.
        It combines the corresponding `genes` of two parent chromosomes by replacing a gene in one chromosome
        with a value that lies between the two parent `genes`. The new gene is calculated as the psudo weighted average
        of the two parent `genes`, where the weight is determined by the `alpha` parameter.

        1. Inputs:
            * Two parent chromosomes (Parent 1 and Parent 2) composed of real-valued genes.
            * A crossover `rate`, determining the probability of applying the operation for each gene.
            * An interpolation parameter (`alpha`), which controls the weight given to each parent's gene during crossover.
        2. Weighted Interpolation:
            * For each gene position in the parents:
            * Generate a random value between 0 and 1.
            * If the random value is less than the rate, compute a new allele as a weighted combination of the parents' alleles:
                new_gene1 = (1 - alpha) * gene_parent1 + alpha * gene_parent2
                new_gene2 = (1 - alpha) * gene_parent2 + alpha * gene_parent1
            * Replace the genes in the offspring with these new values.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param alpha: The interpolation factor that determines the weight of each parent's gene in the new gene value.
        """
        return IntermediateCrossover(rate, alpha)

    @staticmethod
    def shuffle(rate: Rate | float = 0.1):
        """
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

        :param rate: The probability of applying the crossover to a pair of parents.
        """
        return ShuffleCrossover(rate)

    @staticmethod
    def edge_recombination(rate: Rate | float = 0.1):
        """
        The `EdgeRecombinationCrossover` is a specialized crossover operator for permutation problems.
        It focuses on preserving the connectivity between genes by combining edges from both parents.

        1. **Edge List Creation**:
            * For each parent, create a list of edges representing the connections between genes.
        2. **Edge Selection**:
            * Randomly select edges from both parents to create a new offspring.
            * This selection process ensures that the offspring inherits important connections from both parents.
        3. **Child Construction**:
            * The selected edges are used to construct the offspring's gene sequence.
            * This process helps maintain the overall structure and relationships present in the parent solutions.

        **Example**: If Parent 1 has edges (A-B, B-C) and Parent 2 has edges (B-C, C-D), the offspring might inherit edges (A-B, C-D).

        :param rate: The probability of applying the crossover to a pair of parents.
        """
        return EdgeRecombinationCrossover(rate)

    @staticmethod
    def graph(vertex_rate: float = 0.1, edge_rate: float = 0.1):
        """
        This crossover operator is used to combine two parent graphs by swapping the values of their nodes.
        It can be used to create new graphs that inherit the structure and values of their parents.
        Given that a more fit parent's node's `arity` matches the less fit parent's node's `arity`,
        the less fit parent will take (inherit) the more fit parent's node's value. This means the child
        is guaranteed to have the same structure as the less fit parent, but with some of the more fit parent's values (`alleles`).
        This process is extremely similar to how the [NEAT](https://en.wikipedia.org/wiki/NeuroEvolution_of_Augmenting_Topologies) algorithm works.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param parent_node_rate: The probability of inheriting a node's value from the more fit parent.
        """
        return GraphCrossover(vertex_rate, edge_rate)

    @staticmethod
    def tree(rate: Rate | float = 0.1):
        """
        The `TreeCrossover` is a crossover operator that randomly selects a subtree from one parent tree and
        swaps it with a subtree from another parent tree.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param max_size: The maximum size of the resulting tree after crossover.
        """
        return TreeCrossover(rate)


class Mutate:
    @staticmethod
    def uniform(rate: Rate | float = 0.1):
        """
        The most basic mutation operator. It randomly replaces a gene with a new instance of the gene type.

        :param rate: The probability of mutating each gene in an individual.
        """
        return UniformMutator(rate)

    @staticmethod
    def gaussian(rate: Rate | float = 0.1):
        """
        The `GaussianMutator` operator is a mutation mechanism designed for `ArithmeticGene`s.
        It introduces random noise to the gene values by adding a sample from a Gaussian distribution
        with a specified standard deviation. This mutation operator produces small, incremental
        changes centered around the current gene value.

        :param rate: The probability of mutating each gene in an individual.
        """
        return GaussianMutator(rate)

    @staticmethod
    def op(rate: Rate | float = 0.1, replace_rate: float = 0.1):
        """
        This mutator randomly changes or alters the `op` of a node within a `TreeChromosome` or `GraphChromosome`.
        It can replace the `op` with a new one from the store or modify its parameters.

        :param rate: The probability of mutating each gene in an individual.
        :param replace_rate: The probability of replacing the operation entirely instead of just modifying it.
        """
        return OperationMutator(rate, replace_rate)

    @staticmethod
    def graph(
        vertex_rate: float = 0.1, edge_rate: float = 0.1, allow_recurrent: bool = True
    ):
        """
        This mutator is used to add new nodes and connections to the graph.
        It can be used to evolve the graph structure over time, allowing for more complex solutions to emerge.

        :param vertex_rate: The probability of adding a new vertex to the graph.
        :param edge_rate: The probability of adding a new edge to the graph.
        :param allow_recurrent: Whether to allow recurrent connections in the graph.
        """
        return GraphMutator(vertex_rate, edge_rate, allow_recurrent)

    @staticmethod
    def scramble(rate: Rate | float = 0.1):
        """
        The `ScrambleMutator` randomly reorders a segment of `genes` within a `chromosome`.

        :param rate: The probability of mutating each gene in an individual.
        """
        return ScrambleMutator(rate)

    @staticmethod
    def swap(rate: Rate | float = 0.1):
        """
        The `SwapMutator` is a mutation operator designed for genetic algorithms
        to swap the positions of two `Gene`s in a `Chromosome`. This mutator swaps two `Gene`s
        at randomly selected indices, introducing variability while maintaining the `chromosome`s
        structural integrity. It is particularly suited for permutation-based problems.

        :param rate: The probability of mutating each gene in an individual.
        """
        return SwapMutator(rate)

    @staticmethod
    def hoist(rate: Rate | float = 0.1):
        """
        The `HoistMutator` is a mutation operator that randomly selects a subtree
        from the tree and moves it to a different location in the tree. This can create new
        structures and relationships between nodes, allowing for more complex solutions to emerge.

        :param rate: The probability of mutating each gene in an individual.
        """
        return HoistMutator(rate)

    @staticmethod
    def inversion(rate: Rate | float = 0.1):
        """
        `InvertMutator` is a segment inversion mutator. It randomly selects a segment of the
        `chromosome` and inverts the order of the `genes` within that segment.

        :param rate: The probability of mutating each gene in an individual.
        """
        return InversionMutator(rate)

    @staticmethod
    def polynomial(rate: Rate | float = 0.1, eta: float = 20):
        """
        The `PolynomialMutator` applies a polynomial mutation to the genes of a chromosome.
        This provides a bounded and unbiased mutation to genes where you care about the distribution of the mutation.
        Unlike Gaussian mutation, Polynomial can give more control over the tail behavior.

        The `eta` parameter controls the shape of the mutation distribution.
        A higher `eta` value results in a more exploratory mutation, while a lower value makes the
        mutation more exploitative. For example, a low `eta` (1.0-5.0) leads to bigger mutations,
        while a high value (20.0-100.0) leads to smaller, more fine grained mutations.

        :param rate: The probability of mutating each gene in an individual.
        :param eta: The distribution index that controls the shape of the mutation distribution.
        """
        return PolynomialMutator(rate, eta)

    @staticmethod
    def jitter(rate: Rate | float = 0.1, magnitude: float = 0.01):
        """
        The `JitterMutator` adds small random perturbations to the values of a `gene`
        within a `chromosome`. A random value is sampled from a uniform
        distribution between [-1, 1], then it is scaled by the `magnitude`
        parameter and added to the current gene value. This mutation operator is particularly
        useful for fine-tuning solutions in continuous spaces, as it allows for small
        adjustments that can help explore the local neighborhood of a solution.

        :param rate: The probability of mutating each gene in an individual.
        """
        return JitterMutator(rate, magnitude)

    @staticmethod
    def arithmetic(rate: Rate | float = 0.1):
        """
        The `ArithmeticMutator` introduces diversity into genetic algorithms by mutating numerically
        based `genes` through basic arithmetic operations. It is designed to work on `genes` that
        support addition, subtraction, multiplication, and division. Once the values have gone through
        their arithmetic operation, the result is clamped by the `gene`'s bounds to ensure it remains valid.

        1. Choose a random arithmetic operation: addition, subtraction, multiplication, or division.
        2. Apply the operation to the `gene` value using a randomly generated value of the same `gene` type.
        3. Replace the original `gene` with the result of the operation.

        :param rate: The probability of mutating each gene in an individual.
        """
        return ArithmeticMutator(rate)


class Limit:
    @staticmethod
    def score(value: float) -> ScoreLimit:
        return ScoreLimit(value)

    @staticmethod
    def generations(n: int) -> GenerationsLimit:
        return GenerationsLimit(n)

    @staticmethod
    def seconds(secs: int) -> SecondsLimit:
        return SecondsLimit(secs)

    @staticmethod
    def convergence(window: int, threshold: float) -> ConvergenceLimit:
        return ConvergenceLimit(window, threshold)

    @staticmethod
    def metric(
        name: str = "evaluation_count", limit=lambda metric: metric.sum() > 1000
    ):
        return MetricLimit(name, limit)


class Dist:
    @staticmethod
    def euclidean():
        """
        The `EuclideanDistance` is a distance metric that calculates the straight-line distance between two points in a multi-dimensional space.
        It is commonly used in various applications, including clustering, classification, and optimization problems.
        The distance is calculated using the formula:
        """
        return EuclideanDistance()

    @staticmethod
    def cosine():
        """
        The `CosineDistance` is a distance metric that measures the cosine of the angle between two non-zero vectors in a multi-dimensional space.
        It is commonly used in applications such as text analysis and clustering, where the magnitude of the vectors is less important than their orientation. The distance is calculated using the formula:
        D(u, v) = 1 - (u . v) / (||u|| * ||v||)
        where u and v are the two vectors, and ||u|| and ||v|| are their magnitudes.
        """
        return CosineDistance()

    @staticmethod
    def neat(excess: float = 1.0, disjoint: float = 1.0, weight_diff: float = 3.0):
        """
        Initialize the Neat Distance diversity parameter. This follows the same distance metric or algorithm
        described in the NEAT (NeuroEvolution of Augmenting Topologies) algorithm, which is a method for evolving artificial neural networks.
        The distance metric is used to measure the similarity between two neural network genomes, which can be useful for speciation and maintaining diversity in the population.

        :param excess: Excess coefficient.
        :param disjoint: Disjoint coefficient.
        :param weight_diff: Weight difference coefficient.
        """
        return NeatDistance(excess, disjoint, weight_diff)

    @staticmethod
    def hamming():
        """
        Initialize the Hamming Distance diversity parameter. This is a pretty simple distance metric
        that counts the number of differing genes between two chromosomes.
        """
        return HammingDistance()


__all__ = ["Select", "Mutate", "Cross", "Dist", "Limit"]
