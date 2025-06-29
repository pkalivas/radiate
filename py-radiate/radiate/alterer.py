from typing import Dict
from .component import ComponentBase
from .genome.gene import GeneType


class AlterBase(ComponentBase):
    def __init__(
        self,
        component: str,
        args: Dict[str, str] = {},
        allowed_genes: set[GeneType] | GeneType = {},
    ):
        """
        Initialize the base alterer class.
        :param alterer: An instance of the PyAlterer class.
        """
        super().__init__(component=component, args=args)
        if isinstance(allowed_genes, str):
            allowed_genes = {allowed_genes}
        self.allowed_genes = allowed_genes if allowed_genes else GeneType.ALL

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


class BlendCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(
            component="BlendCrossover", args={"rate": str(rate), "alpha": str(alpha)}
        )


class IntermediateCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(
            component="IntermediateCrossover",
            args={"rate": str(rate), "alpha": str(alpha)},
        )


class MeanCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        super().__init__(component="MeanCrossover", args={"rate": str(rate)})


class ShuffleCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="ShuffleCrossover", args={"rate": str(rate)})


class SimulatedBinaryCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, contiguity: float = 0.5):
        super().__init__(
            component="SimulatedBinaryCrossover",
            args={"rate": str(rate), "contiguity": str(contiguity)},
        )


class PartiallyMatchedCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            component="PartiallyMatchedCrossover", args={"rate": str(rate)}
        )


class MultiPointCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, num_points: int = 2):
        super().__init__(
            component="MultiPointCrossover",
            args={"rate": str(rate), "num_points": str(num_points)},
        )


class UniformCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        super().__init__(component="UniformCrossover", args={"rate": str(rate)})


class UniformMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="UniformMutator", args={"rate": str(rate)})


class ArithmeticMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="ArithmeticMutator", args={"rate": str(rate)})


class GaussianMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="GaussianMutator", args={"rate": str(rate)})


class ScrambleMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="ScrambleMutator", args={"rate": str(rate)})


class SwapMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="SwapMutator", args={"rate": str(rate)})


class GraphMutator(AlterBase):
    def __init__(
        self,
        vertex_rate: float = 0.1,
        edge_rate: float = 0.1,
        allow_recurrent: bool = False,
    ):
        super().__init__(
            component="GraphMutator",
            args={
                "vertex_rate": str(vertex_rate),
                "edge_rate": str(edge_rate),
                "allow_recurrent": str(allow_recurrent),
            },
            allowed_genes=GeneType.GRAPH,
        )


class OperationMutator(AlterBase):
    def __init__(self, rate: float = 0.1, replace_rate: float = 0.1):
        super().__init__(
            component="OperationMutator",
            args={"rate": str(rate), "replace_rate": str(replace_rate)},
            allowed_genes=GeneType.GRAPH,
        )


class GraphCrossover(AlterBase):
    def __init__(self, rate: float = 0.5, parent_node_rate: float = 0.5):
        super().__init__(
            component="GraphCrossover",
            args={"rate": str(rate), "parent_node_rate": str(parent_node_rate)},
            allowed_genes=GeneType.GRAPH,
        )


# from radiate.radiate import PyAlterer
# from .genome import Genotype, Chromosome, Population, Phenotype, Gene
# from typing import List


# class AlterBase:
#     def __init__(self, alterer: PyAlterer):
#         """
#         Initialize the base alterer class.
#         :param alterer: An instance of the PyAlterer class.
#         """
#         self.alterer = alterer
#         if not isinstance(alterer, PyAlterer):
#             raise TypeError(f"Expected an instance of PyAlterer, got {type(alterer)}")

#     def __repr__(self):
#         return f"{self.__class__.__name__}(alterer={self.alterer})"

#     def __eq__(self, value):
#         if not isinstance(value, AlterBase):
#             return False
#         return self.alterer == value.alterer

#     def alter(
#         self,
#         population: Population
#         | List[Phenotype]
#         | List[Genotype]
#         | List[Chromosome]
#         | List[Gene],
#     ) -> Population:
#         """
#         Apply the alterer to the provided genotypes or chromosomes.
#         :param genotypes: A list of Genotype or Chromosome instances.
#         :return: A Population instance containing the altered individuals.
#         """
#         population_to_alter = population

#         if all(isinstance(pheno, Phenotype) for pheno in population):
#             population_to_alter = Population(
#                 [phenotype.py_phenotype for phenotype in population_to_alter]
#             )
#         elif all(isinstance(geno, Genotype) for geno in population):
#             phenotypes = [
#                 Phenotype(genotype=genotype.genotype())
#                 for genotype in population_to_alter
#             ]
#             population_to_alter = Population(phenotypes)
#         elif all(isinstance(chromo, Chromosome) for chromo in population):
#             genotypes = [
#                 Genotype(chromosomes=[chromo]) for chromo in population_to_alter
#             ]
#             phenotypes = [Phenotype(genotype=genotype) for genotype in genotypes]
#             population_to_alter = Population(phenotypes)
#         elif all(isinstance(gene, Gene) for gene in population):
#             chromosomes = [Chromosome(genes=[gene]) for gene in population_to_alter]
#             genotypes = [Genotype(chromosomes=chromosomes)]
#             phenotypes = [Phenotype(genotype=genotype) for genotype in genotypes]
#             population_to_alter = Population(phenotypes)
#         elif isinstance(population, Population):
#             population_to_alter = population
#         else:
#             raise TypeError(
#                 f"Expected a Population, list of Phenotypes, Genotypes, Chromosomes, or Genes, got {type(population)}"
#             )

#         altered_population = self.alterer.alter(population_to_alter.py_population())
#         return Population(altered_population)


# class BlendCrossover(AlterBase):
#     def __init__(self, rate: float = 0.1, alpha: float = 0.5):
#         super().__init__(PyAlterer.blend_crossover(rate=rate, alpha=alpha))


# class IntermediateCrossover(AlterBase):
#     def __init__(self, rate: float = 0.1, alpha: float = 0.5):
#         super().__init__(PyAlterer.intermediate_crossover(rate=rate, alpha=alpha))


# class MeanCrossover(AlterBase):
#     def __init__(self, rate: float = 0.5):
#         super().__init__(PyAlterer.mean_crossover(rate=rate))


# class ShuffleCrossover(AlterBase):
#     def __init__(self, rate: float = 0.1):
#         super().__init__(PyAlterer.shuffle_crossover(rate=rate))


# class SimulatedBinaryCrossover(AlterBase):
#     def __init__(self, rate: float = 0.1, contiguty: float = 0.5):
#         super().__init__(
#             PyAlterer.simulated_binary_crossover(rate=rate, contiguty=contiguty)
#         )


# class PartiallyMatchedCrossover(AlterBase):
#     def __init__(self, rate: float = 0.1):
#         super().__init__(PyAlterer.partially_matched_crossover(rate=rate))


# class MultiPointCrossover(AlterBase):
#     def __init__(self, rate: float = 0.1, num_points: int = 2):
#         super().__init__(
#             PyAlterer.multi_point_crossover(rate=rate, num_points=num_points)
#         )


# class UniformCrossover(AlterBase):
#     def __init__(self, rate: float = 0.5):
#         super().__init__(PyAlterer.uniform_crossover(rate=rate))


# class UniformMutator(AlterBase):
#     def __init__(self, rate: float = 0.1):
#         super().__init__(PyAlterer.uniform_mutator(rate=rate))


# class ArithmeticMutator(AlterBase):
#     def __init__(self, rate: float = 0.1):
#         super().__init__(PyAlterer.arithmetic_mutator(rate=rate))


# class GaussianMutator(AlterBase):
#     def __init__(self, rate: float = 0.1):
#         super().__init__(PyAlterer.gaussian_mutator(rate=rate))


# class ScrambleMutator(AlterBase):
#     def __init__(self, rate: float = 0.1):
#         super().__init__(PyAlterer.scramble_mutator(rate=rate))


# class SwapMutator(AlterBase):
#     def __init__(self, rate: float = 0.1):
#         super().__init__(PyAlterer.swap_mutator(rate=rate))


# class GraphMutator(AlterBase):
#     def __init__(
#         self,
#         vertex_rate: float = 0.1,
#         edge_rate: float = 0.1,
#         allow_recurrent: bool = False,
#     ):
#         super().__init__(
#             PyAlterer.graph_mutator(
#                 vertex_rate=vertex_rate,
#                 edge_rate=edge_rate,
#                 allow_recurrent=allow_recurrent,
#             )
#         )


# class OperationMutator(AlterBase):
#     def __init__(self, rate: float = 0.1, replace_rate: float = 0.1):
#         super().__init__(PyAlterer.op_mutator(rate=rate, replace_rate=replace_rate))


# class GraphCrossover(AlterBase):
#     def __init__(self, rate: float = 0.5, parent_node_rate: float = 0.5):
#         super().__init__(
#             PyAlterer.graph_crossover(rate=rate, parent_node_rate=parent_node_rate)
#         )
