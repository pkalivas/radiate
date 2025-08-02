from .base import FitnessBase

from typing import List
from radiate.radiate import PyFitnessFn


class Regression(FitnessBase):
    """Fitness function for regression problems."""

    def __init__(
        self,
        features: List[List[float]],
        targets: List[List[float]],
        loss: str = "mse",
    ):
        """Initialize regression fitness with features, targets, and loss function."""
        if not isinstance(features, List):
            raise TypeError("features must be a list of lists or a pandas DataFrame.")
        if not isinstance(targets, List):
            raise TypeError("targets must be a list of lists or a pandas DataFrame.")

        super().__init__(
            PyFitnessFn.regression(features=features, targets=targets, loss=loss)
        )
