from typing import Any, Callable

from ..operators.distance import DistanceBase, HammingDistance
from .base import FitnessBase
from .custom import BatchFitness, CallableFitness
from .loss import MAE, MSE, Diff, XEnt
from .novelty import NoveltySearch
from .regression import Regression


def fitness[T](batch: bool = False):
    def decorator(func: Callable[[T | list[T]], Any]) -> FitnessBase[T]:
        if batch:
            return BatchFitness(func)
        else:
            return CallableFitness(func)

    return decorator


def novelty[T](
    archive: int = 1000,
    k: int = 15,
    threshold: float = 0.03,
    distance: DistanceBase = HammingDistance(),
):
    def decorator(
        func: Callable[[T], float | int | list[float] | list[int]],
    ) -> NoveltySearch[T]:
        return NoveltySearch(
            descriptor=func,
            archive_size=archive,
            k=k,
            distance=distance,
            threshold=threshold,
        )

    return decorator


__all__ = [
    "FitnessBase",
    "Regression",
    "CallableFitness",
    "BatchFitness",
    "NoveltySearch",
    "fitness",
    "novelty",
    "MSE",
    "MAE",
    "XEnt",
    "Diff",
]
