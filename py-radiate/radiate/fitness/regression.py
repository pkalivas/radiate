from .base import FitnessBase

from radiate.radiate import PyFitnessFn


class Regression[T](FitnessBase[T]):
    """Fitness function for regression problems."""

    def __init__(
        self,
        features: list[list[float]],
        targets: list[list[float]],
        loss: str = "mse",
        batch: bool = False,
    ):
        """Initialize regression fitness with features, targets, and loss function."""
        if not isinstance(features, list):
            raise TypeError("features must be a list of lists.")
        if not isinstance(targets, list):
            raise TypeError("targets must be a list of lists.")

        super().__init__(
            PyFitnessFn.regression(
                features=features, targets=targets, loss=loss, is_batch=batch
            )
        )
