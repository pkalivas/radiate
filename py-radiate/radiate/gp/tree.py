from __future__ import annotations

from typing import TYPE_CHECKING, Any, overload

import numpy as np

from radiate.radiate import PyTree

from .._bridge import RsObject

if TYPE_CHECKING:
    from .._dependancies import pandas as pd
    from .._dependancies import polars as pl


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
        from .._dependancies import _NUMPY_AVAILABLE

        if not _NUMPY_AVAILABLE:
            raise ImportError(
                "NumPy is not available. Please install it to use this feature."
            )
        else:
            from .._dependancies import numpy as np

        input_type = type(inputs).__name__
        eval_data = inputs

        if input_type in ("DataFrame", "Series"):
            if hasattr(inputs, "to_numpy"):  # Pandas / Polars / Backends
                # Optional: Filter by column list if provided
                if columns is not None and hasattr(inputs, "select"):
                    inputs = inputs.select(columns)  # Polars syntax
                elif columns is not None and hasattr(inputs, "__getitem__"):
                    inputs = inputs[columns]  # Pandas syntax

                if not eval_data.flags["C_CONTIGUOUS"]:
                    eval_data = np.ascontiguousarray(eval_data)

                eval_data = inputs.to_numpy()
            else:
                raise TypeError(f"Unsupported dataframe object wrapper: {input_type}")

        return self.__backend__().eval(eval_data)

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
