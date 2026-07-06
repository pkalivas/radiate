from __future__ import annotations

from typing import Any, Dict

from .._rd import components
from ..genome import GeneType
from .base import ComponentBase
from .input import EngineInput, EngineInputType


class DistanceBase(ComponentBase):
    """
    Base class for diversity parameters.
    """

    def __init__(
        self,
        component: str,
        args: Dict[str, Any] = {},
        allowed_genes: set[GeneType] | GeneType = set(GeneType),
    ):
        """
        Initialize the diversity parameter with a PyDiversity instance.
        :param diversity: An instance of PyDiversity.
        """
        super().__init__(component=component, args=args)
        if isinstance(allowed_genes, GeneType):
            allowed_genes = {allowed_genes}
        else:
            allowed_genes = allowed_genes if allowed_genes else GeneType.all()
        self.allowed_genes = allowed_genes

    def __str__(self):
        """
        Return a string representation of the diversity parameter.
        :return: String representation of the diversity parameter.
        """
        return self.__repr__()

    def __repr__(self):
        """
        Return a detailed string representation of the diversity parameter.
        :return: Detailed string representation of the diversity parameter.
        """
        return f"Diversity(component={self.component}, args={self.args}, allowed_genes={self.allowed_genes})"

    def __eq__(self, value):
        if not isinstance(value, DistanceBase):
            return False
        return (
            self.component == value.component
            and self.args == value.args
            and self.allowed_genes == value.allowed_genes
        )


class HammingDistance(DistanceBase):
    """
    Hamming Distance diversity parameter.
    """

    def __init__(self):
        """
        Initialize the Hamming Distance diversity parameter. This is a pretty simiple distance metric
        that counts the number of differing genes between two chromosomes.
        """
        super().__init__(component="HammingDistance")


class EuclideanDistance(DistanceBase):
    """
    Euclidean Distance diversity parameter.
    """

    def __init__(self):
        """
        Initialize the Euclidean Distance diversity parameter.
        """
        super().__init__(component="EuclideanDistance", allowed_genes=GeneType.FLOAT)


class NeatDistance(DistanceBase):
    """
    Neat Distance diversity parameter.
    """

    def __init__(
        self, excess: float = 1.0, disjoint: float = 1.0, weight_diff: float = 3.0
    ):
        """
        Initialize the Neat Distance diversity parameter.
        :param excess: Excess coefficient.
        :param disjoint: Disjoint coefficient.
        :param weight_diff: Weight difference coefficient.
        """
        super().__init__(
            component="NeatDistance",
            args={
                "excess": excess,
                "disjoint": disjoint,
                "weight_diff": weight_diff,
            },
            allowed_genes={GeneType.GRAPH},
        )


class CosineDistance(DistanceBase):
    """
    Cosine Distance diversity parameter.
    """

    def __init__(self):
        """
        Initialize the Cosine Distance diversity parameter.
        """
        super().__init__(component="CosineDistance", allowed_genes=GeneType.FLOAT)


class Dist(EngineInput):
    def __init__(self, component: str, **kwargs):
        super().__init__(
            component=component,
            input_type=EngineInputType.Diversity,
            **kwargs,
        )

    @staticmethod
    def euclidean() -> Dist:
        """
        The `EuclideanDistance` is a distance metric that calculates the straight-line distance between two points in a multi-dimensional space.
        It is commonly used in various applications, including clustering, classification, and optimization problems.
        The distance is calculated using the formula:
        """
        return Dist(
            component=components.EUCLIDEAN_DISTANCE, allowed_genes=GeneType.FLOAT
        )

    @staticmethod
    def cosine() -> Dist:
        """
        The `CosineDistance` is a distance metric that measures the cosine of the angle between two non-zero vectors in a multi-dimensional space.
        It is commonly used in applications such as text analysis and clustering, where the magnitude of the vectors is less important than their orientation. The distance is calculated using the formula:
        D(u, v) = 1 - (u . v) / (||u|| * ||v||)
        where u and v are the two vectors, and ||u|| and ||v|| are their magnitudes.
        """
        return Dist(component=components.COSINE_DISTANCE, allowed_genes=GeneType.FLOAT)

    @staticmethod
    def neat(
        excess: float = 1.0, disjoint: float = 1.0, weight_diff: float = 3.0
    ) -> Dist:
        """
        Initialize the Neat Distance diversity parameter. This follows the same distance metric or algorithm
        described in the NEAT (NeuroEvolution of Augmenting Topologies) algorithm, which is a method for evolving artificial neural networks.
        The distance metric is used to measure the similarity between two neural network genomes, which can be useful for speciation and maintaining diversity in the population.

        :param excess: Excess coefficient.
        :param disjoint: Disjoint coefficient.
        :param weight_diff: Weight difference coefficient.
        """
        return Dist(
            component=components.NEAT_DISTANCE,
            excess=excess,
            disjoint=disjoint,
            weight_diff=weight_diff,
            allowed_genes={GeneType.GRAPH},
        )

    @staticmethod
    def hamming() -> Dist:
        """
        Initialize the Hamming Distance diversity parameter. This is a pretty simple distance metric
        that counts the number of differing genes between two chromosomes.
        """
        return Dist(component=components.HAMMING_DISTANCE, allowed_genes=GeneType.FLOAT)
