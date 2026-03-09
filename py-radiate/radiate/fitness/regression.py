from .base import FitnessBase
from typing import Any
from radiate.radiate import PyFitnessFn
from radiate.utils._normalize import _normalize_regression_data


class Regression(FitnessBase):
    """Fitness function for regression problems."""

    from radiate._typing import RdLossType
    from .loss import MSE

    def __init__(
        self,
        features: Any,
        targets: Any | None = None,
        *,
        target_cols: str | list[str] | None = None,
        feature_cols: list[str] | None = None,
        loss: RdLossType = MSE,
        batch: bool = False,
    ):
        x, y = _normalize_regression_data(
            features,
            targets,
            feature_cols=feature_cols,
            target_cols=target_cols,
        )

        loss_str = str(loss) if loss is not None else None

        super().__init__(
            PyFitnessFn.regression(features=x, targets=y, loss=loss_str, is_batch=batch)
        )
