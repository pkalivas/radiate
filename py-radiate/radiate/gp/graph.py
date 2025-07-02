from typing import List, Tuple, TypeAlias, Dict, Optional, Union
from radiate.gp.op import Op
from radiate.radiate import PyGraph

NodeValues: TypeAlias = Union[List[Op], Op, List[str], str]


class Graph:
    def __init__(
        self,
        py_graph: PyGraph = None,
        shape: Tuple[int, int] = (1, 1),
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]]] = None,
    ):
        if py_graph is not None:
            if not isinstance(py_graph, PyGraph):
                raise TypeError("py_graph must be an instance of PyGraph.")
        else:
            from radiate.codec.graph import GraphCodec

            codec = GraphCodec.directed(
                shape=shape,
                vertex=vertex,
                edge=edge,
                output=output,
                values=values,
            )
            py_graph = codec.decode(codec.encode())
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
