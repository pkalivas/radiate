from typing import Any, Dict

from .component import ComponentBase
from ..genome import GeneType


class DistanceBase(ComponentBase):
    """
    Base class for diversity parameters.
    """

    def __init__(
        self,
        component: str,
        args: Dict[str, Any] = {},
        allowed_genes: set[str] | str = {},
    ):
        """
        Initialize the diversity parameter with a PyDiversity instance.
        :param diversity: An instance of PyDiversity.
        """
        super().__init__(component=component, args=args)
        self.allowed_genes = allowed_genes if allowed_genes else GeneType.all()

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
        Initialize the Hamming Distance diversity parameter.
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
