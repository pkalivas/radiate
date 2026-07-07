from typing import Callable

from radiate.radiate import PyFitnessFn

from ..operators.distance import Dist
from .base import FitnessBase

type NoveltyOutput = float | int | list[float] | list[int]


class NoveltySearch[T](FitnessBase[T]):
    def __init__(
        self,
        distance: Dist | None,
        descriptor: Callable[[T], NoveltyOutput],
        k: int = 15,
        threshold: float = 0.03,
        archive_size: int = 1000,
        batch: bool = False,
    ):
        self._validate_inputs(descriptor, k, threshold, archive_size)

        descriptor = self._setup_descriptor(descriptor, distance)
        distance = self._setup_distance(distance)

        super().__init__(
            PyFitnessFn.novelty_search(
                distance_fn=distance.component,
                descriptor=descriptor,
                k=k,
                threshold=threshold,
                archive_size=archive_size,
                is_batch=batch,
            )
        )

    def _validate_inputs(
        self,
        descriptor: Callable[[T], NoveltyOutput],
        k: int,
        threshold: float,
        archive_size: int,
    ):
        if not isinstance(descriptor, Callable):
            raise TypeError(
                "descriptor must be a callable or an instance of DescriptorBase."
            )

        if k <= 0:
            raise ValueError("k must be a positive integer.")
        if threshold < 0:
            raise ValueError("threshold must be a non-negative float.")
        if archive_size <= 0:
            raise ValueError("archive_size must be a positive integer.")

    def _setup_descriptor(
        self, descriptor: Callable[[T], NoveltyOutput], distance: Dist | None
    ):
        if isinstance(descriptor, Callable):
            if distance is None:
                distance = Dist.euclidean()
        return descriptor

    def _setup_distance(self, distance: Dist | None) -> Dist:
        if distance is None:
            # HammingDistance works with all gene types
            distance = Dist.hamming()
        return distance
