from __future__ import annotations
from radiate.radiate import PyGraph


class Graph:
    def __init__(self, pygraph: PyGraph):
        if not isinstance(pygraph, PyGraph):
            raise TypeError("pygraph must be an instance of PyGraph")
        self.inner = pygraph

    def __repr__(self):
        return self.inner.__repr__()

    def __str__(self):
        return self.inner.__str__()

    def __eq__(self, other):
        if not isinstance(other, Graph):
            return False
        return self.inner == other.inner

    def eval(self, inputs: list[list[float]] | list[float]) -> list[list[float]]:
        return self.inner.eval(inputs)

    def reset(self):
        self.inner.reset()

    def to_dot(self) -> str:
        return self.inner.to_dot()

    def to_json(self) -> str:
        return self.inner.to_json()

    @staticmethod
    def from_json(json_str: str) -> Graph:
        return Graph(PyGraph.from_json(json_str))
