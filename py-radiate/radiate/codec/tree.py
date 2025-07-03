from typing import List, Optional, Dict, TypeAlias, Union, Tuple

from .codec import CodecBase
from radiate.gp import Op, Tree
from radiate.genome import Genotype
from radiate.radiate import PyTreeCodec

NodeValues: TypeAlias = Union[List[Op], Op, List[str], str]


class TreeCodec(CodecBase):
    def encode(self) -> "Genotype":
        return Genotype(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> "Tree":
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        if genotype.gene_type() != "TreeNode":
            raise ValueError("genotype must be of type 'tree'.")
        return Tree(self.codec.decode_py(genotype.py_genotype()))

    def __init__(
        self,
        shape: Tuple[int, int] = (1, 1),
        min_depth: int = 3,
        max_size: int = 30,
        vertex: Optional[NodeValues] = None,
        leaf: Optional[NodeValues] = None,
        root: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]] | List[Tuple[str, List[Op]]]] = None,
    ) -> "TreeCodec":
        input_size, output_size = shape

        if input_size < 1 or output_size < 1:
            raise ValueError("Input and output size must be at least 1")
        if min_depth < 1:
            raise ValueError("Minimum depth must be at least 1")
        if max_size < 1:
            raise ValueError("Maximum size must be at least 1")

        ops_map: Dict[str, List[Op]] = {}
        if leaf is None and values is None:
            ops_map = {"leaf": [Op.var(i) for i in range(input_size)]}
        if values is not None:
            if isinstance(values, list):
                values = dict(values)
            ops_map = values | ops_map
        else:
            if vertex is not None:
                ops_map["vertex"] = [vertex] if isinstance(vertex, Op) else vertex
            if root is not None:
                ops_map["root"] = [root] if isinstance(root, Op) else root
            if leaf is not None:
                ops_map["leaf"] = [leaf] if isinstance(leaf, Op) else leaf

        self.codec = PyTreeCodec(output_size, min_depth, max_size, ops_map)
