from .base import FitnessBase

from typing import Callable

from radiate.inputs.descriptor import CustomDescriptor, DescriptorBase
from radiate.inputs.distance import DistanceBase, EuclideanDistance, HammingDistance
from radiate.radiate import PyFitnessFn


class NoveltySearch[T](FitnessBase[T]):
    """Fitness function for novelty search algorithms."""

    def __init__(
        self,
        distance: DistanceBase | None,
        descriptor: Callable[[T], float | list[float]] | DescriptorBase,
        k: int = 15,
        threshold: float = 0.03,
        archive_size: int = 1000,
        batch: bool = False,
    ):
        """Initialize novelty search with descriptor, distance function, and parameters."""
        self._validate_inputs(descriptor, distance, k, threshold, archive_size)

        descriptor = self._setup_descriptor(descriptor, distance)
        distance = self._setup_distance(distance)

        super().__init__(
            PyFitnessFn.novelty_search(
                distance_fn=distance.component,
                descriptor=descriptor.descriptor,
                k=k,
                threshold=threshold,
                archive_size=archive_size,
                is_batch=batch,
            )
        )

    def _validate_inputs(self, descriptor, distance, k, threshold, archive_size):
        """Validate constructor inputs."""
        if not isinstance(descriptor, (Callable, DescriptorBase)):
            raise TypeError(
                "descriptor must be a callable or an instance of DescriptorBase."
            )
        if distance is not None and not isinstance(distance, DistanceBase):
            raise TypeError("distance must be an instance of DistanceBase.")
        if k <= 0:
            raise ValueError("k must be a positive integer.")
        if threshold < 0:
            raise ValueError("threshold must be a non-negative float.")
        if archive_size <= 0:
            raise ValueError("archive_size must be a positive integer.")

    def _setup_descriptor(self, descriptor, distance):
        """Setup descriptor with appropriate distance function."""
        if isinstance(descriptor, Callable):
            descriptor = CustomDescriptor(descriptor)
            if distance is None:
                distance = EuclideanDistance()
        return descriptor

    def _setup_distance(self, distance):
        """Setup distance function with default fallback."""
        if distance is None:
            # HammingDistance works with all gene types
            distance = HammingDistance()
        return distance
