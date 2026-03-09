from .base import FitnessBase
from typing import Any
from radiate.radiate import PyFitnessFn
from radiate.utils._normalize import _normalize_regression_data


class Regression(FitnessBase):
    """Fitness function for regression problems."""

    def __init__(
        self,
        features: Any,
        targets: Any | None = None,
        *,
        target_cols: str | list[str] | None = None,
        feature_cols: list[str] | None = None,
        loss: str = "mse",
        batch: bool = False,
    ):
        x, y = _normalize_regression_data(
            features,
            targets,
            feature_cols=feature_cols,
            target_cols=target_cols,
        )

        super().__init__(
            PyFitnessFn.regression(features=x, targets=y, loss=loss, is_batch=batch)
        )
