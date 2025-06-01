from radiate.radiate import PyAlterer


class AlterBase:
    def __init__(self, alterer: PyAlterer):
        """
        Initialize the base alterer class.
        :param alterer: An instance of the PyAlterer class.
        """
        self.alterer = alterer
        if not isinstance(alterer, PyAlterer):
            raise TypeError(f"Expected an instance of PyAlterer, got {type(alterer)}")

    def __repr__(self):
        return f"{self.__class__.__name__}(alterer={self.alterer})"


class BlendCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(PyAlterer.blend_crossover(rate=rate, alpha=alpha))


class IntermediateCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        super().__init__(PyAlterer.intermediate_crossover(rate=rate, alpha=alpha))


class MeanCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        super().__init__(PyAlterer.mean_crossover(rate=rate))


class ShuffleCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(PyAlterer.shuffle_crossover(rate=rate))


class SimulatedBinaryCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, contiguty: float = 0.5):
        super().__init__(
            PyAlterer.simulated_binary_crossover(rate=rate, contiguty=contiguty)
        )


class PartiallyMatchedCrossover(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(PyAlterer.partially_matched_crossover(rate=rate))


class MultiPointCrossover(AlterBase):
    def __init__(self, rate: float = 0.1, num_points: int = 2):
        super().__init__(
            PyAlterer.multi_point_crossover(rate=rate, num_points=num_points)
        )


class UniformCrossover(AlterBase):
    def __init__(self, rate: float = 0.5):
        super().__init__(PyAlterer.uniform_crossover(rate=rate))


class UniformMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(PyAlterer.uniform_mutator(rate=rate))


class ArithmeticMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(PyAlterer.arithmetic_mutator(rate=rate))


class GaussianMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(PyAlterer.gaussian_mutator(rate=rate))


class ScrambleMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(PyAlterer.scramble_mutator(rate=rate))


class SwapMutator(AlterBase):
    def __init__(self, rate: float = 0.1):
        super().__init__(PyAlterer.swap_mutator(rate=rate))
