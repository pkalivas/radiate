from typing import List, TypeAlias, Union
from radiate.gp.op import Op
from radiate.radiate import PyGraph

NodeValues: TypeAlias = Union[List[Op], Op, List[str], str]


class Graph:
    def __init__(
        self,
        py_graph: PyGraph = None,
    ):
        if py_graph is not None:
            if not isinstance(py_graph, PyGraph):
                raise TypeError("py_graph must be an instance of PyGraph.")
        self.py_graph = py_graph

    def __repr__(self):
        return self.py_graph.__repr__()

    def __len__(self):
        return len(self.py_graph)

    def __eq__(self, other):
        if not isinstance(other, Graph):
            return False
        return self.py_graph == other.py_graph

    def eval(
        self, inputs: List[float] | List[List[float]]
    ) -> List[float] | List[List[float]]:
        if isinstance(inputs, list) and all(
            isinstance(i, (float, int)) for i in inputs
        ):
            inputs = [inputs]
        elif not isinstance(inputs, list) or not all(
            isinstance(i, list) for i in inputs
        ):
            raise ValueError(
                "Inputs must be a list of floats or a list of list of floats"
            )
        return self.py_graph.eval(inputs)
