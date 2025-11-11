from .base import FitnessBase
from .regression import Regression
from .custom import CallableFitness, BatchFitness
from .novelty import NoveltySearch

__all__ = [
    "FitnessBase",
    "Regression",
    "CallableFitness",
    "BatchFitness",
    "NoveltySearch",
]
