from .base import FitnessBase

from typing import Any, Callable

from radiate.radiate import PyFitnessFn


class CallableFitness[T](FitnessBase[T]):
    """Wrapper for user-defined callable fitness functions."""

    def __init__(self, problem: Callable[[T], Any]):
        """Initialize with a callable fitness function."""
        super().__init__(PyFitnessFn.custom(problem))
