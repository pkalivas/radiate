from __future__ import annotations

from radiate.radiate import PyTree
from radiate.wrapper import PyObject


class Tree(PyObject[PyTree]):
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

    def eval(self, inputs: list[list[float]]) -> list[list[float]]:
        """
        Evaluate the tree with the given inputs. The inputs needs to be a list of
        lists (for multiple samples).

        Args:
            inputs (list[list[float]]): The input data to evaluate the tree on.
        Returns:
            list[list[float]]: The output of the tree after evaluation.
        """
        return self.__backend__().eval(inputs)

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
