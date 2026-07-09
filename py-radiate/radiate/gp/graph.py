from __future__ import annotations

from typing import TYPE_CHECKING, Any, overload

from radiate.radiate import PyGraph

from .._bridge import RsObject
from ..genome.chromosome import Chromosome
from ..utils._normalize import _to_float_array

if TYPE_CHECKING:
    from .._dependancies import numpy as np
    from .._dependancies import pandas as pd
    from .._dependancies import polars as pl


class Graph(RsObject):
    @classmethod
    def from_chromosome(cls, chromosome: Chromosome) -> Graph:
        return cls.from_rust(PyGraph.from_chromosome(chromosome.__backend__()))

    def __repr__(self):
        return self.__backend__().__repr__()

    def __str__(self):
        return self.__backend__().__str__()

    def __eq__(self, other):
        if not isinstance(other, Graph):
            return False
        return self.__backend__() == other.__backend__()

    def __len__(self):
        return self.__backend__().__len__()

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
        self,
        inputs: "np.typing.NDArray[np.float32]",
    ) -> (
        "np.typing.NDArray[np.float32]"
    ): ...  # Performance upgrade: return array to array users

    @overload
    def eval(
        self,
        inputs: "np.typing.NDArray[np.float64]",
    ) -> (
        "np.typing.NDArray[np.float64]"
    ): ...  # Performance upgrade: return array to array users

    @overload
    def eval(
        self, inputs: "pl.DataFrame | pl.Series", *, columns: list[str] | None = None
    ) -> "np.ndarray": ...

    @overload
    def eval(
        self, inputs: "pd.DataFrame | pd.Series", *, columns: list[str] | None = None
    ) -> "np.ndarray": ...

    def eval(
        self, inputs: Any, *, columns: list[str] | None = None
    ) -> list[list[float]] | list[float] | "np.ndarray":
        """Evaluate the graph with the given inputs.

        Supports 1D/2D Lists, NumPy arrays, Polars, and Pandas objects.
        """
        graph_shape = self.shape()
        eval_data = _to_float_array(inputs, columns=columns)

        shape = eval_data.shape
        dims = len(shape)
        if dims == 1:
            if graph_shape[0] != shape[0]:
                raise ValueError(
                    f"Input length {shape[0]} does not match graph input size {graph_shape[0]}"
                )
        elif dims == 2:
            if graph_shape[0] != shape[1]:
                raise ValueError(
                    f"Input width {shape[1]} does not match graph input size {graph_shape[0]}"
                )

        return self.__backend__().eval(eval_data)

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
