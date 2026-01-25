from .base import FitnessBase
from .regression import Regression
from .custom import CallableFitness, BatchFitness
from .novelty import NoveltySearch
from functools import wraps
from typing import Any, Callable
from radiate.inputs.distance import DistanceBase, HammingDistance


def fitness(
    func: Callable[..., Any] | None = None,
    /,
    *,
    batch: bool = False,
):
    def decorator(f: Callable[..., Any]) -> Callable[..., Any]:
        @wraps(f)
        def wrapper(*args, **kwargs):
            return f(*args, **kwargs)

        return BatchFitness(wrapper) if batch else CallableFitness(wrapper)

    return decorator if func is None else decorator(func)


def novelty(
    behavior_func: Callable[..., Any] | None = None,
    /,
    *,
    archive: int = 1000,
    k: int = 15,
    threshold: float = 0.03,
    distance: DistanceBase = HammingDistance(),
):
    def decorator(f: Callable[..., Any]) -> Callable[..., Any]:
        @wraps(f)
        def wrapper(*args, **kwargs):
            return f(*args, **kwargs)

        return NoveltySearch(
            descriptor=wrapper,
            archive_size=archive,
            k=k,
            distance=distance,
            threshold=threshold,
        )

    return decorator if behavior_func is None else decorator(behavior_func)


__all__ = [
    "FitnessBase",
    "Regression",
    "CallableFitness",
    "BatchFitness",
    "NoveltySearch",
    "fitness",
    "novelty",
]
