from typing import Any

from radiate.genome.population import Population
from radiate.inputs.input import EngineInput, EngineInputType
from .component import ComponentBase
from ..genome import GeneType


class AlterBase(ComponentBase):
    def __init__(
        self,
        component: str,
        args: dict[str, Any] = {},
        allowed_genes: set[GeneType] | GeneType = set(),
    ):
        """
        Initialize the base alterer class.
        :param alterer: An instance of the PyAlterer class.
        """
        super().__init__(component=component, args=args)
        self.allowed_genes = allowed_genes if allowed_genes else GeneType.core()

    def __repr__(self):
        return f"{self.__class__.__name__}(alterer={self.component}, args={self.args}, allowed_genes={self.allowed_genes})"

    def __eq__(self, value):
        if not isinstance(value, AlterBase):
            return False
        return (
            self.component == value.component
            and self.args == value.args
            and self.allowed_genes == value.allowed_genes
        )

    def alter(self, population: Population, generation: int = 0):
        """
        Alter the population based on the alterer's criteria.
        :param population: The population to alter.
        :param generation: The current generation number.
        :return: The altered population.
        """
        from radiate.radiate import py_alter

        alterer_input = EngineInput(
            component=self.component,
            input_type=EngineInputType.Alterer,
            allowed_genes=self.allowed_genes,
            args=self.args,
        ).__backend__()

        return Population(
            individuals=py_alter(
                population.__backend__().gene_type(),
                alterer_input,
                population.__backend__(),
                generation,
            )
        )


class BlendCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        """
        The `BlendCrossover` is a crossover operator designed for `ArithmeticGene`s (IntGene & FloatGene).
        It introduces variability by blending the `gene` controlled by the `alpha` parameter.
        This approach allows for smooth transitions between `gene` values, promoting exploration of the search space.
        Its functionality is similar to the `IntermediateCrossover`, but it uses a different formula to calculate the new `gene` value.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param alpha: The blending factor that determines the extent of blending between parent `gene` values.
        """
        super().__init__(
            component="BlendCrossover",
            args={"rate": rate, "alpha": alpha},
            allowed_genes=GeneType.FLOAT,
        )


class IntermediateCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
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
        super().__init__(
            component="IntermediateCrossover",
            args={"rate": rate, "alpha": alpha},
            allowed_genes=GeneType.FLOAT,
        )


class MeanCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        """
        The `MeanCrossover` operator is a crossover mechanism designed
        for `ArithmeticGene`s. It combines the corresponding `genes` of two parent chromosomes by
        replacing a gene in the first parent with the mean (average) of the two `genes`. This approach
        is useful when `genes` represent numeric values such as weights or coordinates,
        as it promotes a balanced combination of parent traits.

        :param rate: The probability of applying the crossover to a pair of parents.
        """
        super().__init__(
            component="MeanCrossover",
            args={"rate": rate},
            allowed_genes={GeneType.FLOAT, GeneType.INT, GeneType.ANY},
        )


class ShuffleCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
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
        super().__init__(component="ShuffleCrossover", args={"rate": rate})


class SimulatedBinaryCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, contiguity: float = 0.5):
        """
        The `SimulatedBinaryCrossover` is a crossover operator designed for `FloatGene`s.
        It simulates binary crossover by creating offspring that are a linear combination of the parents, controlled
        by a contiguity factor. Effectively, it allows for a smooth transition between parent values
        while maintaining the overall structure of the `genes` by sampling from a uniform distribution between the two parents.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param contiguity: The contiguity factor that influences the distribution of offspring `gene` values between the parents.
        """
        super().__init__(
            component="SimulatedBinaryCrossover",
            args={"rate": rate, "contiguity": contiguity},
            allowed_genes=GeneType.FLOAT,
        )


class PartiallyMappedCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
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
        super().__init__(
            component="PartiallyMappedCrossover",
            args={"rate": rate},
            allowed_genes=GeneType.PERMUTATION,
        )


class MultiPointCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, num_points: int = 2):
        """
        The `MultiPointCrossover` is a crossover operator that combines two parent individuals by selecting
        multiple crossover points and swapping the genetic material between the parents at those points. This is a
        classic crossover operator.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param num_points: The number of crossover points to use.
        """
        super().__init__(
            component="MultiPointCrossover",
            args={"rate": rate, "num_points": num_points},
        )


class UniformCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        """
        The `UniformCrossover` is a crossover operator creates new individuals by selecting `genes`
        from the parents with equal probability and swapping them between the parents.
        This is a simple crossover operator that can be effective in a wide range of problems.

        :param rate: The probability of applying the crossover to a pair of parents.
        """
        super().__init__(component="UniformCrossover", args={"rate": rate})


class UniformMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        """
        The most basic mutation operator. It randomly replaces a gene with a new instance of the gene type.

        :param rate: The probability of mutating each gene in an individual.
        """
        super().__init__(component="UniformMutator", args={"rate": rate})


class ArithmeticMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
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
        super().__init__(
            component="ArithmeticMutator",
            args={"rate": rate},
            allowed_genes={GeneType.FLOAT, GeneType.INT, GeneType.ANY},
        )


class GaussianMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        """
        The `GaussianMutator` operator is a mutation mechanism designed for `ArithmeticGene`s.
        It introduces random noise to the gene values by adding a sample from a Gaussian distribution
        with a specified standard deviation. This mutation operator produces small, incremental
        changes centered around the current gene value.

        :param rate: The probability of mutating each gene in an individual.
        """
        super().__init__(
            component="GaussianMutator",
            args={"rate": rate},
            allowed_genes={GeneType.FLOAT, GeneType.INT},
        )


class ScrambleMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        """
        The `ScrambleMutator` randomly reorders a segment of `genes` within a `chromosome`.

        :param rate: The probability of mutating each gene in an individual.
        """
        super().__init__(component="ScrambleMutator", args={"rate": rate})


class SwapMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        """
        The `SwapMutator` is a mutation operator designed for genetic algorithms
        to swap the positions of two `Gene`s in a `Chromosome`. This mutator swaps two `Gene`s
        at randomly selected indices, introducing variability while maintaining the `chromosome`s
        structural integrity. It is particularly suited for permutation-based problems.

        :param rate: The probability of mutating each gene in an individual.
        """
        super().__init__(component="SwapMutator", args={"rate": rate})


class GraphMutator(AlterBase):
    def __init__(
        self,
        vertex_rate: float = 0.1,
        edge_rate: float = 0.1,
        allow_recurrent: bool = True,
    ):
        """
        This mutator is used to add new nodes and connections to the graph.
        It can be used to evolve the graph structure over time, allowing for more complex solutions to emerge.

        :param vertex_rate: The probability of adding a new vertex to the graph.
        :param edge_rate: The probability of adding a new edge to the graph.
        :param allow_recurrent: Whether to allow recurrent connections in the graph.
        """
        super().__init__(
            component="GraphMutator",
            args={
                "vertex_rate": vertex_rate,
                "edge_rate": edge_rate,
                "allow_recurrent": allow_recurrent,
            },
            allowed_genes=GeneType.GRAPH,
        )


class OperationMutator(AlterBase):
    def __init__(self, rate: float = 0.1, replace_rate: float = 0.1):
        """
        This mutator randomly changes or alters the `op` of a node within a `TreeChromosome` or `GraphChromosome`.
        It can replace the `op` with a new one from the store or modify its parameters.

        :param rate: The probability of mutating each gene in an individual.
        :param replace_rate: The probability of replacing the operation entirely instead of just modifying it.
        """
        super().__init__(
            component="OperationMutator",
            args={"rate": rate, "replace_rate": replace_rate},
            allowed_genes={GeneType.GRAPH, GeneType.TREE},
        )


class GraphCrossover(AlterBase):
    def __init__(self, rate: float = 0.5, parent_node_rate: float = 0.5):
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
        super().__init__(
            component="GraphCrossover",
            args={"rate": rate, "parent_node_rate": parent_node_rate},
            allowed_genes=GeneType.GRAPH,
        )


class TreeCrossover(AlterBase):
    def __init__(self, rate: float = 0.5, max_size: int = 30):
        """
        The `TreeCrossover` is a crossover operator that randomly selects a subtree from one parent tree and
        swaps it with a subtree from another parent tree.

        :param rate: The probability of applying the crossover to a pair of parents.
        :param max_size: The maximum size of the resulting tree after crossover.
        """
        super().__init__(
            component="TreeCrossover",
            args={"rate": rate, "max_size": max_size},
            allowed_genes=GeneType.TREE,
        )


class HoistMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        """
        The `HoistMutator` is a mutation operator that randomly selects a subtree
        from the tree and moves it to a different location in the tree. This can create new
        structures and relationships between nodes, allowing for more complex solutions to emerge.

        :param rate: The probability of mutating each gene in an individual.
        """
        super().__init__(
            component="HoistMutator",
            args={"rate": rate},
            allowed_genes=GeneType.TREE,
        )


class InversionMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        """
        `InvertMutator` is a segment inversion mutator. It randomly selects a segment of the
        `chromosome` and inverts the order of the `genes` within that segment.

        :param rate: The probability of mutating each gene in an individual.
        """
        super().__init__(
            component="InversionMutator",
            args={"rate": rate},
        )


class EdgeRecombinationCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
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
        super().__init__(
            component="EdgeRecombinationCrossover",
            args={"rate": rate},
            allowed_genes=GeneType.PERMUTATION,
        )


class PolynomialMutator(AlterBase):
    def __init__(self, rate: float = 0.5, eta: float = 20.0):
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
        super().__init__(
            component="PolynomialMutator",
            args={"rate": rate, "eta": eta},
            allowed_genes=GeneType.FLOAT,
        )


class JitterMutator(AlterBase):
    def __init__(self, rate: float = 0.1, magnitude: float = 0.1):
        """
        The `JitterMutator` adds small random perturbations to the values of a `gene`
        within a `chromosome`. A random value is sampled from a uniform
        distribution between [-1, 1], then it is scaled by the `magnitude`
        parameter and added to the current gene value. This mutation operator is particularly
        useful for fine-tuning solutions in continuous spaces, as it allows for small
        adjustments that can help explore the local neighborhood of a solution.

        :param rate: The probability of mutating each gene in an individual.
        """
        super().__init__(
            component="JitterMutator",
            args={"rate": rate, "magnitude": magnitude},
            allowed_genes=GeneType.FLOAT,
        )
