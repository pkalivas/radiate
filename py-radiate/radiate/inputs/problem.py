import abc
from typing import List, Callable, Any
from radiate.genome.gene import GeneType
from radiate.inputs.descriptor import CustomDescriptor, DescriptorBase
from radiate.inputs.input import EngineInput, EngineInputType
from radiate.inputs.distance import (
    DistanceBase,
    EuclideanDistance,
    GraphArchitectureDistance,
    GraphTopologyDistance,
    HammingDistance,
)
from radiate.radiate import PyProblemBuilder, PyNoveltySearchFitnessBuilder


class ProblemBase(abc.ABC):
    """A class representing a problem to be solved using evolutionary algorithms."""

    def __init__(self, problem: PyProblemBuilder):
        self.problem = problem


class CallableProblem(ProblemBase):
    """A class representing a custom problem defined by the user."""

    def __init__(self, problem: Callable[[Any], Any]):
        """
        Initializes the CallableProblem instance.

        :param problem: A callable defining the custom problem.
        """
        super().__init__(PyProblemBuilder.custom(problem))


class Regression(ProblemBase):
    """A class representing a regression problem."""

    def __init__(
        self,
        features: List[List[float]],
        targets: List[List[float]],
        loss: str = "mse",
    ):
        """
        Initializes the Regression problem instance.

        :param codec: An instance of CodecBase used for encoding/decoding genotypes.
        :param data: A list of tuples where each tuple contains input features and the corresponding target value.
        :param kwargs: Additional keyword arguments for problem configuration.
        """
        if not isinstance(features, List):
            raise TypeError("features must be a list of lists or a pandas DataFrame.")
        if not isinstance(targets, List):
            raise TypeError("targets must be a list of lists or a pandas DataFrame.")

        super().__init__(
            PyProblemBuilder.regression(features=features, targets=targets, loss=loss)
        )


class NoveltySearch(ProblemBase):
    """A class representing a novelty search problem."""

    def __init__(
        self,
        distance: DistanceBase | None,
        descriptor: Callable[[Any], float | List[float]] | DescriptorBase,
        k: int = 15,
        threshold: float = 0.03,
        archive_size: int = 1000,
    ):
        """
        Initializes the NoveltySearch problem instance.

        :param features: A list of feature vectors.
        :param targets: A list of target vectors.
        :param k: The number of nearest neighbors to consider for novelty search.
        :param threshold: The novelty threshold.
        """
        if not isinstance(descriptor, (Callable, DescriptorBase)):
            raise TypeError(
                "descriptor must be a callable or an instance of DescriptorBase."
            )

        if isinstance(descriptor, Callable):
            descriptor = CustomDescriptor(descriptor)
            if distance is None:
                distance = EuclideanDistance()

        # Default distance if not provided - HammingDistance is the
        # only distance that works with all gene types.
        if distance is None:
            distance = HammingDistance()

        if not isinstance(distance, DistanceBase):
            raise TypeError("distance must be an instance of DistanceBase.")
        if k <= 0:
            raise ValueError("k must be a positive integer.")
        if threshold < 0:
            raise ValueError("threshold must be a non-negative float.")
        if archive_size <= 0:
            raise ValueError("archive_size must be a positive integer.")

        builder = PyNoveltySearchFitnessBuilder(
            descriptor=descriptor.descriptor
            if isinstance(descriptor, CustomDescriptor)
            else descriptor,
            distance=EngineInput(
                input_type=EngineInputType.Diversity,
                component=distance.component,
                allowed_genes=distance.allowed_genes
                if not descriptor
                else GeneType.ALL,
                **distance.args,
            ).py_input(),
            k=k,
            threshold=threshold,
            archive_size=archive_size,
        )

        input = EngineInput(
            input_type=EngineInputType.Diversity,
            component=distance.component,
            allowed_genes=distance.allowed_genes if not descriptor else GeneType.ALL,
            **distance.args,
        ).py_input()

        super().__init__(
            PyProblemBuilder.novelty_search(
                distance=input,
                descriptor=descriptor,
                k=k,
                threshold=threshold,
                archive_size=archive_size,
                is_native=isinstance(
                    distance, (GraphTopologyDistance, GraphArchitectureDistance)
                ),
            )
        )
