from typing import Any

from .graph import Graph
from .tree import Tree
from radiate.utils._normalize import _normalize_regression_data
from radiate._bridge.wrapper import RsObject
from radiate.radiate import py_accuracy


class AccuracyResult(RsObject):
    def __repr__(self) -> str:
        return self.__backend__().__repr__()

    def __str__(self) -> str:
        return self.__backend__().__str__()

    def name(self) -> str:
        return self.__backend__().name()

    def accuracy(self) -> float:
        return self.__backend__().accuracy()

    def precision(self) -> float | None:
        return self.__backend__().precision()

    def recall(self) -> float | None:
        return self.__backend__().recall()

    def f1_score(self) -> float | None:
        return self.__backend__().f1_score()

    def rmse(self) -> float | None:
        return self.__backend__().rmse()

    def r_squared(self) -> float | None:
        return self.__backend__().r_squared()

    def sample_count(self) -> int:
        return self.__backend__().sample_count()

    def loss(self) -> float | None:
        return self.__backend__().loss()

    def loss_name(self) -> str | None:
        return self.__backend__().loss_fn()


def accuracy(
    predictor: Graph | Tree,
    features: Any,
    targets: Any,
    loss: str | None = None,
    name: str | None = None,
) -> AccuracyResult:
    """
    Calculate the accuracy of a predictor (Graph or Tree) against given features and targets.

    Args:
        predictor (Graph | Tree): The predictor to evaluate.
        features (list[list[float]]): The input features.
        targets (list[list[float]]): The expected target outputs.
        loss (str | None): The loss function to use. Defaults to None.
        name (str | None): An optional name for the accuracy metric. Defaults to None.

    Returns:
        AccuracyResult: The calculated accuracy result.
    """
    if not isinstance(predictor, (Graph, Tree)):
        raise TypeError(
            f"predictor must be an instance of Graph or Tree but found {type(predictor)}."
        )

    x, y = _normalize_regression_data(features, targets)

    accuracy_result = py_accuracy(
        predictor.__backend__(),
        x,
        y,
        loss=loss,
        name=name,
    )

    return AccuracyResult.from_rust(accuracy_result)
