from __future__ import annotations

from radiate.radiate import PyTree


class Tree:
    def __init__(self, pytree: PyTree):
        if not isinstance(pytree, PyTree):
            raise TypeError("pytree must be an instance of PyTree")
        self.inner = pytree

    def __repr__(self):
        return self.inner.__repr__()

    def __str__(self):
        return self.inner.__str__()

    def __eq__(self, other):
        if not isinstance(other, Tree):
            return False
        return self.inner == other

    def __len__(self):
        return len(self.inner)

    def eval(self, inputs: list[list[float]]) -> list[list[float]]:
        return self.inner.eval(inputs)

    def to_dot(self) -> str:
        return self.inner.to_dot()

    def to_json(self) -> str:
        return self.inner.to_json()

    @staticmethod
    def from_json(json_str: str) -> Tree:
        return Tree(PyTree.from_json(json_str))
