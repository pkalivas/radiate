from .base import FitnessBase

from typing import Any, Callable

from radiate.radiate import PyFitnessFn


class CallableFitness(FitnessBase):
    """Wrapper for user-defined callable fitness functions."""

    def __init__(self, problem: Callable[[Any], Any]):
        """Initialize with a callable fitness function."""
        super().__init__(PyFitnessFn.custom(problem))
