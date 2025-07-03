from typing import List, Tuple, Dict, Optional, TypeAlias, Union
from radiate.gp.op import Op
from radiate.radiate import PyTree

NodeValues: TypeAlias = Union[List[Op], Op, List[str], str]


class Tree:
    def __init__(
        self,
        py_tree: PyTree = None,
        min_depth: int = 3,
        max_size: int = 30,
        vertex: Optional[NodeValues] = None,
        leaf: Optional[NodeValues] = None,
        root: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]] | List[Tuple[str, List[Op]]]] = None,
    ):
        if py_tree is not None:
            if not isinstance(py_tree, PyTree):
                raise TypeError("py_tree must be an instance of PyTree.")
            else:
                self.py_tree = py_tree
        else:
            from radiate.codec.tree import TreeCodec

            codec = TreeCodec(
                vertex=vertex,
                leaf=leaf,
                root=root,
                values=values,
                min_depth=min_depth,
                max_size=max_size,
            )

            self.py_tree = codec.decode(codec.encode())

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
