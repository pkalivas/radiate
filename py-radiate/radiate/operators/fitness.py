from __future__ import annotations

from collections.abc import Callable
from functools import wraps
from typing import Any, Literal, TypeVar, overload

from radiate._typing import RdLossType

from .._rd import PyFitnessFn
from ..dsl.loss import MSE
from ..utils._normalize import _normalize_regression_data
from .distance import Dist
from .input import EngineInput, EngineInputType

type NoveltyOutput = float | int | list[float] | list[int]


T = TypeVar("T")


@overload
def fitness(func: Callable[[T | list[T]], Any], /) -> Fitness[T]: ...


@overload
def fitness(
    *, batch: Literal[False] = False
) -> Callable[[Callable[[T], Any]], Fitness[T]]: ...


@overload
def fitness(
    *, batch: Literal[True] = True
) -> Callable[[Callable[[list[T]], Any]], Fitness[T]]: ...


def fitness(
    func: Callable[[T | list[T]], Any] | None = None, *, batch: bool = False
) -> Any:
    def decorator(f: Callable[[T | list[T]], Any]) -> Fitness[T]:
        @wraps(f)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            return f(*args, **kwargs)

        if batch:
            return Fitness.custom(fitness_fn=wrapper, is_batch=True)
        return Fitness.custom(fitness_fn=wrapper, is_batch=False)

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
    ) -> Fitness[T]:
        return Fitness.novelty(
            descriptor=func,
            archive_size=archive,
            k=k,
            distance=distance,
            threshold=threshold,
        )

    return decorator


class Fitness[T](EngineInput):
    def __init__(self, **kwargs):
        super().__init__(input_type=EngineInputType.FitnessFunction, **kwargs)

    @staticmethod
    def custom(fitness_fn: Callable[[T], object], is_batch: bool = False) -> Fitness:
        if not isinstance(fitness_fn, Callable):
            raise TypeError("fitness_fn must be a callable.")
        return Fitness(fitness=PyFitnessFn.custom(fitness_fn, is_batch=is_batch))

    @staticmethod
    def novelty(
        descriptor: Callable[[T], NoveltyOutput],
        distance: Dist | None = None,
        k: int = 15,
        threshold: float = 0.03,
        archive_size: int = 1000,
        is_batch: bool = False,
    ) -> Fitness:
        _validate_inputs(descriptor, k, threshold, archive_size)

        descriptor = _setup_descriptor(descriptor, distance)
        distance_fn = _setup_distance(distance)

        if distance_fn.component is None:
            raise ValueError("Distance function must be provided for novelty search.")

        return Fitness(
            fitness=PyFitnessFn.novelty_search(
                distance_fn=distance_fn.component,
                descriptor=descriptor,
                k=k,
                threshold=threshold,
                archive_size=archive_size,
                is_batch=is_batch,
            )
        )

    @staticmethod
    def regression(
        features: Any,
        targets: Any | None = None,
        *,
        target_cols: str | list[str] | None = None,
        feature_cols: list[str] | None = None,
        loss: RdLossType = MSE,
        batch: bool = False,
    ) -> Fitness:
        x, y = _normalize_regression_data(
            features,
            targets,
            feature_cols=feature_cols,
            target_cols=target_cols,
        )

        loss_str = str(loss) if loss is not None else str(MSE)

        return Fitness(
            fitness=PyFitnessFn.regression(
                features=x,
                targets=y,
                loss=loss_str,
                is_batch=batch,
            )
        )


def _validate_inputs[T](
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


def _setup_descriptor[T](
    descriptor: Callable[[T], NoveltyOutput], distance: Dist | None
):
    if isinstance(descriptor, Callable):
        if distance is None:
            distance = Dist.euclidean()
    return descriptor


def _setup_distance(distance: Dist | None) -> Dist:
    if distance is None:
        distance = Dist.hamming()
    return distance
