from __future__ import annotations

from typing import TYPE_CHECKING, Any, overload

from radiate.radiate import PyGraph

from .._bridge import RsObject
from ..genome.chromosome import Chromosome

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
        self, inputs: "np.ndarray", *, columns: list[str] | None = None
    ) -> "np.ndarray": ...  # Performance upgrade: return array to array users

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
        from .._dependancies import _NUMPY_AVAILABLE

        if not _NUMPY_AVAILABLE:
            raise ImportError(
                "NumPy is not available. Please install it to use this feature."
            )
        else:
            from .._dependancies import numpy as np

        input_type = type(inputs).__name__
        graph_shape = self.shape()
        eval_data = inputs

        if input_type in ("DataFrame", "Series"):
            if hasattr(inputs, "to_numpy"):
                # Polars syntax
                if columns is not None and hasattr(inputs, "select"):
                    inputs = inputs.select(columns)
                    eval_data = inputs.to_numpy(order="c")

                # Pandas syntax
                elif columns is not None and hasattr(inputs, "__getitem__"):
                    inputs = inputs[columns]
                    eval_data = inputs.to_numpy()

            else:
                raise TypeError(f"Unsupported dataframe object wrapper: {input_type}")

            if not eval_data.flags["C_CONTIGUOUS"]:
                eval_data = np.ascontiguousarray(eval_data)

        if hasattr(eval_data, "shape"):
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

        # Rust handles dimension tracking and f64 -> f32 downcasting internally.
        # It always yields a Bound<'py, PyArrayDyn<f32>> back to the CPython layer.
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
