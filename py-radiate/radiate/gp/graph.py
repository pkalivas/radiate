from __future__ import annotations

from typing import Any, overload, TYPE_CHECKING

from radiate.radiate import PyGraph
from radiate._bridge.wrapper import RsObject
from radiate.utils import _normalize_single_chunk

if TYPE_CHECKING:
    from radiate._dependancies import numpy as np
    from radiate._dependancies import polars as pl
    from radiate._dependancies import pandas as pd


class Graph(RsObject):
    def __repr__(self):
        return self.__backend__().__repr__()

    def __str__(self):
        return self.__backend__().__str__()

    def __eq__(self, other):
        if not isinstance(other, Graph):
            return False
        return self.__backend__() == other.__backend__()

    def shape(self) -> tuple[int, int]:
        """
        Get the shape of the graph in terms of number of input and output nodes.

        Returns:
            tuple[int, int]: A tuple containing the number of input nodes and output nodes in the graph.
        """
        return self.__backend__().shape()

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

    def reset(self):
        """
        Reset the internal state of the graph. For graphs with recurrent connections, the internal
        state keeps track of previous node evaluations. This method clears that state and essentially
        restarts the graph evaluation from scratch.
        """
        self.__backend__().reset()

    def to_dot(self) -> str:
        """
        Convert the graph to DOT format for visualization. This representation can be used with graph
        visualization tools like Graphviz.

        Returns:
            str: The DOT format string representing the graph.
        """
        return self.__backend__().to_dot()

    def to_json(self) -> str:
        """
        Serialize the graph to a JSON string. Pretty simple.

        Returns:
            str: The JSON string representation of the graph.
        """
        return self.__backend__().to_json()

    @staticmethod
    def from_json(json_str: str) -> Graph:
        """
        Deserialize a JSON string to a Graph object.

        Args:
            json_str (str): The JSON string representation of the graph.
        Returns:
            Graph: The deserialized Graph object.
        """
        return Graph.from_rust(PyGraph.from_json(json_str))
