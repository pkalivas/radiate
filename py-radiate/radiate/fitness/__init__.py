from functools import wraps
from typing import Any, Callable, Literal, TypeVar, overload

from ..operators.distance import Dist
from .base import FitnessBase
from .custom import BatchFitness, CallableFitness
from .loss import MAE, MSE, Diff, XEnt
from .novelty import NoveltySearch
from .regression import Regression

T = TypeVar("T")


@overload
def fitness(func: Callable[[T | list[T]], Any], /) -> FitnessBase[T]: ...


@overload
def fitness(
    *, batch: Literal[False] = False
) -> Callable[[Callable[[T], Any]], FitnessBase[T]]: ...


@overload
def fitness(
    *, batch: Literal[True] = True
) -> Callable[[Callable[[list[T]], Any]], FitnessBase[T]]: ...


def fitness(
    func: Callable[[T | list[T]], Any] | None = None, *, batch: bool = False
) -> Any:
    def decorator(f: Callable[[T | list[T]], Any]) -> FitnessBase[T]:
        @wraps(f)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            return f(*args, **kwargs)

        if batch:
            return BatchFitness(wrapper)
        return CallableFitness(wrapper)

    if func is not None:
        return decorator(func)
    return decorator


def novelty[T](
    archive: int = 1000,
    k: int = 15,
    threshold: float = 0.03,
    distance: Dist = Dist.hamming(),
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
