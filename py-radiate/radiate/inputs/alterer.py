from typing import Any, Dict

from radiate.genome.population import Population
from radiate.inputs.input import EngineInput, EngineInputType
from .component import ComponentBase
from ..genome.gene import GeneType


class AlterBase(ComponentBase):
    def __init__(
        self,
        component: str,
        args: Dict[str, Any] = {},
        allowed_genes: set[GeneType] | GeneType = {},
    ):
        """
        Initialize the base alterer class.
        :param alterer: An instance of the PyAlterer class.
        """
        super().__init__(component=component, args=args)
        if isinstance(allowed_genes, str):
            allowed_genes = {allowed_genes}
        self.allowed_genes = allowed_genes if allowed_genes else GeneType.CORE

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

    def alter(self, population, generation: int = 0):
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
        ).py_input()

        return Population(individuals=py_alter(
            population.py_population().gene_type(),
            alterer_input,
            population.py_population(),
            generation,
        ))


class BlendCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(
            component="BlendCrossover",
            args={"rate": rate, "alpha": alpha},
            allowed_genes=GeneType.FLOAT,
        )


class IntermediateCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(
            component="IntermediateCrossover",
            args={"rate": rate, "alpha": alpha},
            allowed_genes=GeneType.FLOAT,
        )


class MeanCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        super().__init__(component="MeanCrossover", args={"rate": rate})


class ShuffleCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="ShuffleCrossover", args={"rate": rate})


class SimulatedBinaryCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, contiguity: float = 0.5):
        super().__init__(
            component="SimulatedBinaryCrossover",
            args={"rate": rate, "contiguity": contiguity},
        )


class PartiallyMappedCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            component="PartiallyMappedCrossover",
            args={"rate": rate},
            allowed_genes=GeneType.PERMUTATION,
        )


class MultiPointCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, num_points: int = 2):
        super().__init__(
            component="MultiPointCrossover",
            args={"rate": rate, "num_points": num_points},
        )


class UniformCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        super().__init__(component="UniformCrossover", args={"rate": rate})


class UniformMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="UniformMutator", args={"rate": rate})


class ArithmeticMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="ArithmeticMutator", args={"rate": rate})


class GaussianMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="GaussianMutator", args={"rate": rate})


class ScrambleMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="ScrambleMutator", args={"rate": rate})


class SwapMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(component="SwapMutator", args={"rate": rate})


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
                "vertex_rate": vertex_rate,
                "edge_rate": edge_rate,
                "allow_recurrent": allow_recurrent,
            },
            allowed_genes=GeneType.GRAPH,
        )


class OperationMutator(AlterBase):
    def __init__(self, rate: float = 0.1, replace_rate: float = 0.1):
        super().__init__(
            component="OperationMutator",
            args={"rate": rate, "replace_rate": replace_rate},
            allowed_genes={GeneType.GRAPH, GeneType.TREE},
        )


class GraphCrossover(AlterBase):
    def __init__(self, rate: float = 0.5, parent_node_rate: float = 0.5):
        super().__init__(
            component="GraphCrossover",
            args={"rate": rate, "parent_node_rate": parent_node_rate},
            allowed_genes=GeneType.GRAPH,
        )


class TreeCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        super().__init__(
            component="TreeCrossover",
            args={"rate": rate},
            allowed_genes=GeneType.TREE,
        )


class HoistMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            component="HoistMutator",
            args={"rate": rate},
            allowed_genes=GeneType.TREE,
        )


class InversionMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            component="InversionMutator",
            args={"rate": rate},
        )
