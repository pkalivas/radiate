from __future__ import annotations

from collections.abc import Callable
from enum import StrEnum
from functools import wraps
from typing import TYPE_CHECKING, Any, Literal, Protocol, overload

from radiate._typing import RdDataType, RdLossType

from .._rd import PyFitnessFn
from ..dsl.loss import MSE
from ..utils._normalize import _normalize_regression_data
from .distance import Dist
from .input import EngineInput, EngineInputType

if TYPE_CHECKING:
    from radiate._dependancies import numpy as np

type NoveltyOutput = float | int | list[float] | list[int]


class _BatchFitnessDecorator(Protocol):
    """What `fitness(batch=True)` returns. A numpy batch arrives as one array, and
    numpy engines decode to `np.ndarray`, so it is typed `Fitness[np.ndarray]`. A list
    batch is typed by its element type."""

    @overload
    def __call__(self, func: Callable[[np.ndarray], Any], /) -> Fitness[np.ndarray]: ...
    @overload
    def __call__[U](self, func: Callable[[list[U]], Any], /) -> Fitness[U]: ...


@overload
def fitness[U](func: Callable[[U], Any], /) -> Fitness[U]: ...


@overload
def fitness[U](
    *, batch: Literal[False] = False
) -> Callable[[Callable[[U], Any]], Fitness[U]]: ...


@overload
def fitness(*, batch: Literal[True]) -> _BatchFitnessDecorator: ...


def fitness(
    func: Callable[..., Any] | None = None,
    *,
    batch: bool = False,
) -> Any:
    def decorator(f: Callable[..., Any]) -> Fitness[Any]:
        @wraps(f)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            return f(*args, **kwargs)

        if batch:
            return Fitness.custom(fitness_fn=wrapper, is_batch=True)
        return Fitness.custom(fitness_fn=wrapper, is_batch=False)

    if func is not None:
        return decorator(func)
    return decorator


class _NoveltyDecorator(Protocol):
    """What `novelty(...)` returns. The decorated descriptor's parameter type is only
    known when the decorator is applied, so it is bound here rather than on `novelty`."""

    def __call__[U](self, func: Callable[[U], NoveltyOutput], /) -> Fitness[U]: ...


def novelty(
    archive: int = 1000,
    k: int = 15,
    threshold: float = 0.03,
    distance: Dist = Dist.hamming(),
) -> _NoveltyDecorator:
    def decorator(func: Callable[[Any], NoveltyOutput]) -> Fitness[Any]:
        return Fitness.novelty(
            descriptor=func,
            archive_size=archive,
            k=k,
            distance=distance,
            threshold=threshold,
        )

    return decorator


class FitnessType(StrEnum):
    CUSTOM = "custom"
    NOVELTY = "novelty"
    REGRESSION = "regression"


class Fitness[T](EngineInput):
    def __init__(self, fitness_type: FitnessType, **kwargs):
        super().__init__(input_type=EngineInputType.FitnessFunction, **kwargs)
        self.fitness_type = fitness_type

    @overload
    @staticmethod
    def custom[U](
        fitness_fn: Callable[[U], object], is_batch: Literal[False] = False
    ) -> Fitness[U]: ...

    @overload
    @staticmethod
    def custom(
        fitness_fn: Callable[[np.ndarray], object], is_batch: Literal[True]
    ) -> Fitness[np.ndarray]: ...

    @overload
    @staticmethod
    def custom[U](
        fitness_fn: Callable[[list[U]], object], is_batch: Literal[True]
    ) -> Fitness[U]: ...

    @staticmethod
    def custom(
        fitness_fn: Callable[..., object], is_batch: bool = False
    ) -> Fitness[Any]:
        if not isinstance(fitness_fn, Callable):
            raise TypeError("fitness_fn must be a callable.")
        return Fitness(
            FitnessType.CUSTOM,
            fitness=PyFitnessFn.custom(fitness_fn, is_batch=is_batch),
        )

    @staticmethod
    def novelty(
        descriptor: Callable[[T], NoveltyOutput],
        distance: Dist | None = None,
        k: int = 15,
        threshold: float = 0.03,
        archive_size: int = 1000,
        is_batch: bool = False,
    ) -> Fitness[Any]:
        _validate_inputs(descriptor, k, threshold, archive_size)

        descriptor = _setup_descriptor(descriptor, distance)
        distance_fn = _setup_distance(distance)

        if distance_fn.component is None:
            raise ValueError("Distance function must be provided for novelty search.")

        return Fitness(
            FitnessType.NOVELTY,
            fitness=PyFitnessFn.novelty_search(
                distance_fn=distance_fn.component,
                descriptor=descriptor,
                k=k,
                threshold=threshold,
                archive_size=archive_size,
                is_batch=is_batch,
            ),
        )

    @staticmethod
    def regression(
        dtype: RdDataType,
        features: Any,
        targets: Any | None = None,
        *,
        target_cols: str | list[str] | None = None,
        feature_cols: list[str] | None = None,
        loss: RdLossType = MSE,
    ) -> Fitness[Any]:
        x, y = _normalize_regression_data(
            features,
            targets,
            feature_cols=feature_cols,
            target_cols=target_cols,
        )

        loss_str = str(loss) if loss is not None else str(MSE)

        return Fitness(
            FitnessType.REGRESSION,
            fitness=PyFitnessFn.regression(
                dtype=dtype,
                features=x,
                targets=y,
                loss=loss_str,
            ),
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
