import abc
from typing import List
from radiate.radiate import PyProblemBuilder


class ProblemBase(abc.ABC):
    """A class representing a problem to be solved using evolutionary algorithms."""

    def __init__(self, problem: PyProblemBuilder):
        self.problem = problem


class Regression(ProblemBase):
    """A class representing a regression problem."""

    def __init__(self, features: List[List[float]], targets: List[List[float]], loss: str = "mse"):
        """
        Initializes the Regression problem instance.

        :param codec: An instance of CodecBase used for encoding/decoding genotypes.
        :param data: A list of tuples where each tuple contains input features and the corresponding target value.
        :param kwargs: Additional keyword arguments for problem configuration.
        """
        super().__init__(
            PyProblemBuilder.regression(features=features, targets=targets, loss=loss)
        )
