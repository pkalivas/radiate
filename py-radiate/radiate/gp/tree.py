from typing import List, TypeAlias, Union
from radiate.gp.op import Op
from radiate.radiate import PyTree

NodeValues: TypeAlias = Union[List[Op], Op, List[str], str]


class Tree:
    def __init__(
        self,
        py_tree: PyTree = None,
    ):
        if py_tree is not None:
            if not isinstance(py_tree, PyTree):
                raise TypeError("py_tree must be an instance of PyTree.")
        self.py_tree = py_tree

    def __repr__(self):
        return self.py_tree.__repr__()

    def __len__(self):
        return len(self.py_tree)

    def __eq__(self, other):
        if not isinstance(other, Tree):
            return False
        return self.py_tree == other.py_tree

    def eval(
        self, inputs: List[float] | List[List[float]]
    ) -> List[float] | List[List[float]]:
        if isinstance(inputs, list) and all(
            isinstance(i, (float, int)) for i in inputs
        ):
            inputs = [inputs]
        if not isinstance(inputs, list) or not all(isinstance(i, list) for i in inputs):
            raise ValueError(
                "Inputs must be a list of floats or a list of list of floats"
            )
        return self.py_tree.eval(inputs)
