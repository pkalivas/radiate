from ._typing import GeneType
from .param import EngineParam
from typing import Dict, List

from radiate.radiate import Alterer as Operator

class AlterTemp:
    def __init__(self, operator: Operator):
        """
        Initialize the alterer with an operator.
        :param operator: Operator to be used for the alterer.
        """
        self.operator = operator

    def __repr__(self):
        """
        String representation of the alterer.
        :return: String representation of the alterer.
        """
        return f"{self.__class__.__name__}(operator={self.operator})"
    
    def __str__(self):
        """
        String representation of the alterer.
        :return: String representation of the alterer.
        """
        return f"{self.__class__.__name__}(operator={self.operator})"

class BlendCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(
            operator=Operator.blend_crossover(rate=rate, alpha=alpha),
        )

class IntermediateCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(
            operator=Operator.intermediate_crossover(rate=rate, alpha=alpha),
        )

class MeanCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.5):
        super().__init__(
            operator=Operator.mean_crossover(rate=rate),
        )

class ShuffleCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            operator=Operator.shuffle_crossover(rate=rate),
        )

class SimulatedBinaryCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.1, contiguty: float = 0.5):
        super().__init__(
            operator=Operator.simulated_binary_crossover(rate=rate, contiguty=contiguty),
        )

class PartiallyMatchedCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            operator=Operator.partially_matched_crossover(rate=rate),
        )

class MultiPointCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.1, num_points: int = 2):
        super().__init__(
            operator=Operator.multi_point_crossover(rate=rate, num_points=num_points),
        )

class UniformCrossoverTemp(AlterTemp):
    def __init__(self, rate: float = 0.5):
        super().__init__(
            operator=Operator.uniform_crossover(rate=rate),
        )


class UniformMutatorTemp(AlterTemp):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            operator=Operator.uniform_mutator(rate=rate),
        )

class ArithmeticMutatorTemp(AlterTemp):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            operator=Operator.arithmetic_mutator(rate=rate),
        )

class GaussianMutatorTemp(AlterTemp):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            operator=Operator.gaussian_mutator(rate=rate),
        )

class ScrambleMutatorTemp(AlterTemp):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            operator=Operator.scramble_mutator(rate=rate),
        )

class SwapMutatorTemp(AlterTemp):
    def __init__(self, rate: float = 0.1):
        super().__init__(
            operator=Operator.swap_mutator(rate=rate),
        )


class Alterer(EngineParam):
    """
    Base class for all alterers.
    """

    gene_types: List[GeneType] = []
    name: str = None

    def __init__(
        self, name: str, args: Dict[str, any] = None, gene_types: List[GeneType] = None
    ):
        """
        Initialize the alterer.
        :param name: Name of the alterer.
        :param args: Arguments for the alterer.
        :param gene_types: List of gene types for the alterer.
        """
        super().__init__(name=name, args={k: str(v) for k, v in args.items()})
        self.gene_types = gene_types or self.gene_types

    def __getattr__(self, name):
        """
        Get the value of an attribute.
        :param name: Name of the attribute.
        :return: Value of the attribute.
        """
        if name in self.args:
            return self.args[name]
        if name in self.__dict__:
            return self.__dict__[name]
        raise AttributeError(
            f"'{self.__class__.__name__}' object has no attribute '{name}'"
        )

    def is_valid(self, gene_type: GeneType):
        """
        Validate the gene type.
        :param gene_type: Gene type to validate.
        :return: True if the gene type is valid, False otherwise.
        """
        if not isinstance(gene_type, GeneType):
            raise TypeError(
                f"Gene type {type(gene_type)} is not supported. Expected GeneType."
            )
        return gene_type in self.gene_types


class BlendCrossover(Alterer):
    """
    Blend Crossover alterer.
    """

    gene_types = [GeneType.FLOAT()]
    name = "blend_crossover"

    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        """
        Initialize the blend crossover alterer.
        :param alpha: Alpha value for the blend crossover.
        :param rate: Rate of crossover.
        """
        super().__init__(
            name=self.name,
            args={"rate": rate, "alpha": alpha},
            gene_types=self.gene_types,
        )


class IntermediateCrossover(Alterer):
    """
    Intermediate Crossover alterer.
    """

    gene_types = [GeneType.FLOAT(), GeneType.INT()]
    name = "intermediate_crossover"

    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        """
        Initialize the intermediate crossover alterer.
        :param alpha: Alpha value for the intermediate crossover.
        """
        super().__init__(
            name=self.name,
            args={"rate": rate, "alpha": alpha},
            gene_types=self.gene_types,
        )


class MeanCrossover(Alterer):
    """
    Mean Crossover alterer.
    """

    gene_types = [GeneType.FLOAT(), GeneType.INT()]
    name = "mean_crossover"

    def __init__(self, rate: float = 0.5):
        """
        Initialize the mean crossover alterer.
        :param alpha: Alpha value for the mean crossover.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class ShuffleCrossover(Alterer):
    """
    Shuffle Crossover alterer.
    """

    gene_types = [GeneType.INT(), GeneType.CHAR(), GeneType.BIT(), GeneType.FLOAT()]
    name = "shuffle_crossover"

    def __init__(self, rate: float = 0.1):
        """
        Initialize the shuffle crossover alterer.
        :param alpha: Alpha value for the shuffle crossover.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class SimulatedBinaryCrossover(Alterer):
    """
    Simulated Binary Crossover alterer.
    """

    gene_types = [GeneType.FLOAT()]
    name = "simulated_binary_crossover"

    def __init__(self, rate: float = 0.1, contiguty: float = 0.5):
        """
        Initialize the simulated binary crossover alterer.
        :param alpha: Alpha value for the simulated binary crossover.
        """
        super().__init__(
            name=self.name,
            args={"rate": rate, "contiguty": contiguty},
            gene_types=self.gene_types,
        )


class PartiallyMatchedCrossover(Alterer):
    """
    Partially Matched Crossover alterer.
    """

    gene_types = [GeneType.INT(), GeneType.CHAR(), GeneType.BIT(), GeneType.FLOAT()]
    name = "partially_matched_crossover"

    def __init__(self, rate: float = 0.1):
        """
        Initialize the partially matched crossover alterer.
        :param alpha: Alpha value for the partially matched crossover.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class MultiPointCrossover(Alterer):
    """
    Multi Point Crossover alterer.
    """

    gene_types = [GeneType.INT(), GeneType.CHAR(), GeneType.BIT(), GeneType.FLOAT()]
    name = "multi_point_crossover"

    def __init__(self, rate: float = 0.1, num_points: int = 2):
        """
        Initialize the multi point crossover alterer.
        :param alpha: Alpha value for the multi point crossover.
        """
        super().__init__(
            name=self.name,
            args={"rate": rate, "num_points": num_points},
            gene_types=self.gene_types,
        )


class UniformCrossover(Alterer):
    """
    Uniform Crossover alterer.
    """

    gene_types = [GeneType.INT(), GeneType.CHAR(), GeneType.BIT(), GeneType.FLOAT()]
    name = "uniform_crossover"

    def __init__(self, rate: float = 0.5):
        """
        Initialize the uniform crossover alterer.
        :param alpha: Alpha value for the uniform crossover.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class UniformMutator(Alterer):
    """
    Uniform Mutator alterer.
    """

    gene_types = [GeneType.INT(), GeneType.CHAR(), GeneType.BIT(), GeneType.FLOAT()]
    name = "uniform_mutator"

    def __init__(self, rate: float = 0.1):
        """
        Initialize the uniform mutator alterer.
        :param rate: Rate of mutation.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class ArithmeticMutator(Alterer):
    """
    Arithmetic Mutator alterer.
    """

    gene_types = [GeneType.FLOAT(), GeneType.INT()]
    name = "arithmetic_mutator"

    def __init__(self, rate: float = 0.1):
        """
        Initialize the arithmetic mutator alterer.
        :param alpha: Alpha value for the arithmetic mutator.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class GaussianMutator(Alterer):
    """
    Gaussian Mutator alterer.
    """

    gene_types = [GeneType.FLOAT()]
    name = "gaussian_mutator"

    def __init__(self, rate: float = 0.1):
        """
        Initialize the gaussian mutator alterer.
        :param alpha: Alpha value for the gaussian mutator.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class ScrambleMutator(Alterer):
    """
    Scramble Mutator alterer.
    """

    gene_types = [GeneType.INT(), GeneType.CHAR(), GeneType.BIT(), GeneType.FLOAT()]
    name = "scramble_mutator"

    def __init__(self, rate: float = 0.1):
        """
        Initialize the scramble mutator alterer.
        :param alpha: Alpha value for the scramble mutator.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )


class SwapMutator(Alterer):
    """
    Swap Mutator alterer.
    """

    gene_types = [GeneType.INT(), GeneType.CHAR(), GeneType.BIT(), GeneType.FLOAT()]
    name = "swap_mutator"

    def __init__(self, rate: float = 0.1):
        """
        Initialize the swap mutator alterer.
        :param alpha: Alpha value for the swap mutator.
        """
        super().__init__(
            name=self.name, args={"rate": rate}, gene_types=self.gene_types
        )
