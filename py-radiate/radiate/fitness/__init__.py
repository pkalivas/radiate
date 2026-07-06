from functools import wraps
from typing import Any, Callable

from ..operators.distance import DistanceBase, HammingDistance
from .base import FitnessBase
from .custom import BatchFitness, CallableFitness
from .loss import MAE, MSE, Diff, XEnt
from .novelty import NoveltySearch
from .regression import Regression


def fitness(
    func: Callable[..., Any] | None = None,
    /,
    *,
    batch: bool = False,
):
    def decorator(f: Callable[..., Any]) -> FitnessBase:
        @wraps(f)
        def wrapper(*args, **kwargs):
            return f(*args, **kwargs)

        return BatchFitness(wrapper) if batch else CallableFitness(wrapper)

    return decorator if func is None else decorator(func)


def novelty[T](
    archive: int = 1000,
    k: int = 15,
    threshold: float = 0.03,
    distance: DistanceBase = HammingDistance(),
):
    def decorator(func: Callable[[T], float | list[float]]) -> NoveltySearch[T]:
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
