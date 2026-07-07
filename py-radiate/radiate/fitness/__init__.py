from functools import wraps
from typing import Any, Callable, Literal, TypeVar, overload

from ..dsl.loss import MAE, MSE, Diff, XEnt
from ..operators.distance import Dist
from ..operators.fitness import Fitness
from .base import FitnessBase
from .custom import BatchFitness, CallableFitness
from .novelty import NoveltySearch
from .regression import Regression

__all__ = [
    "FitnessBase",
    "Regression",
    "CallableFitness",
    "BatchFitness",
    "NoveltySearch",
    "novelty",
    "MSE",
    "MAE",
    "XEnt",
    "Diff",
]
