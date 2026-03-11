from __future__ import annotations
from typing import TYPE_CHECKING, overload, Any

from radiate.radiate import PyTree
from radiate._bridge.wrapper import RsObject
from radiate.utils import _normalize_single_chunk

if TYPE_CHECKING:
    from radiate._dependancies import numpy as np
    from radiate._dependancies import polars as pl
    from radiate._dependancies import pandas as pd


class Tree(RsObject):
    def __repr__(self):
        return self.__backend__().__repr__()

    def __str__(self):
        return self.__backend__().__str__()

    def __eq__(self, other):
        if not isinstance(other, Tree):
            return False
        return self.__backend__() == other.__backend__()

    def __len__(self):
        """
        The Tree object actually holds a set of trees (a forest). This method returns the number of trees
        in that forest. We store the trees this way to allow for multiple outputs from a single tree structure.

        Returns:
            int: The number of trees in the forest.
        """
        return len(self.__backend__())

    @overload
    def eval(
        self, inputs: list[list[float]], *, columns: list[str] | None = None
    ) -> list[list[float]]: ...

    @overload
    def eval(
        self, inputs: list[float], *, columns: list[str] | None = None
    ) -> list[float]: ...

    @overload
    def eval(
        self, inputs: "np.ndarray", *, columns: list[str] | None = None
    ) -> list[float]: ...

    @overload
    def eval(
        self, inputs: "pl.DataFrame | pl.Series", *, columns: list[str] | None = None
    ) -> list[list[float]]: ...

    @overload
    def eval(
        self, inputs: "pd.DataFrame | pd.Series", *, columns: list[str] | None = None
    ) -> list[list[float]]: ...

    def eval(
        self, inputs: Any, *, columns: list[str] | None = None
    ) -> list[list[float]] | list[float]:
        """
        Evaluate the graph with the given inputs. The inputs needs to be a list of
        lists (for multiple samples).

        Args:
            inputs (list[list[float]] | list[float]): The input data to evaluate the graph on.
        Returns:
            list[list[float]] | list[float]: The output of the graph after evaluation.
        """
        if isinstance(inputs, list) and all(
            isinstance(row, (int, float)) for row in inputs
        ):
            return self.__backend__().eval(inputs)

        eval_inputs = _normalize_single_chunk(inputs, cols=columns)
        return self.__backend__().eval(eval_inputs)

    def to_dot(self) -> str:
        """
        Convert the tree to DOT format for visualization. This representation can be used with graph
        visualization tools like Graphviz.

        Returns:
            str: The DOT format string representing the tree.
        """
        return self.__backend__().to_dot()

    def to_json(self) -> str:
        """
        Serialize the tree to a JSON string. Pretty simple.

        Returns:
            str: The JSON string representation of the tree.
        """
        return self.__backend__().to_json()

    @staticmethod
    def from_json(json_str: str) -> Tree:
        """
        Deserialize a JSON string to a Tree object.

        Args:
            json_str (str): The JSON string representation of the tree.
        Returns:
            Tree: The deserialized Tree object.
        """
        return Tree.from_rust(PyTree.from_json(json_str))
