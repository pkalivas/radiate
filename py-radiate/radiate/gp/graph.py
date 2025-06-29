from typing import List
from radiate.radiate import PyGraph

class Graph:

    def __init__(self, py_graph: PyGraph):
        self.py_graph = py_graph

    def __repr__(self):
        return self.py_graph.__repr__()
    
    def eval(self, inputs: List[float] | List[List[float]]) -> List[float] | List[List[float]]:
        if isinstance(inputs, list) and all(isinstance(i, (float, int)) for i in inputs):
            inputs = [inputs]
        elif not isinstance(inputs, list) or not all(isinstance(i, list) for i in inputs):
            raise ValueError("Inputs must be a list of floats or a list of list of floats")
        return self.py_graph.eval(inputs)