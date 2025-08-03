from __future__ import annotations

from typing import List, Optional, Dict, Tuple

from radiate._typing import NodeValues
from radiate.genome.gene import GeneType

from .base import CodecBase
from radiate.gp import Op, Tree

from radiate.genome import Genotype
from radiate.radiate import PyTreeCodec


class TreeCodec(CodecBase[Op, Tree]):
    def encode(self) -> Genotype[Op]:
        return Genotype.from_python(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> Tree:
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        if genotype.gene_type() != GeneType.TREE:
            raise ValueError(f"genotype must be of type {genotype.gene_type()}.")
        return Tree(self.codec.decode_py(genotype=genotype.to_python()))

    def __init__(
        self,
        shape: Tuple[int, int] = (1, 1),
        min_depth: int = 3,
        max_size: int = 30,
        vertex: Optional[NodeValues] = None,
        leaf: Optional[NodeValues] = None,
        root: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]] | List[Tuple[str, List[Op]]]] = None,
    ) -> TreeCodec[Op, Tree]:
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
