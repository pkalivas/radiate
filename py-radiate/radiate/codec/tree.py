from __future__ import annotations

from radiate.radiate import PyTreeCodec

from .._bridge import RsObject
from .._typing import AtLeastOne
from ..dsl.dtype import DataType, DataTypeClass, Float64
from ..genome import GeneType, Genotype
from ..gp import Op, OpsConfig, Tree
from ..gp.op import OpBuilder
from .base import CodecBase


class TreeCodec(CodecBase[Op, Tree], RsObject):
    gene_type = GeneType.TREE

    def __init__(
        self,
        shape: tuple[int, int] = (1, 1),
        min_depth: int = 3,
        max_size: int = 30,
        vertex: Op | list[Op] | None = None,
        leaf: Op | list[Op] | None = None,
        root: Op | list[Op] | None = None,
        dtype: DataTypeClass | DataType = Float64,
    ):
        """
        Initialize a TreeCodec for genetic programming trees. The codec supports building trees with
        specified operations for vertices, leaves, and roots. The trees can be constrained by minimum depth and maximum size.

        Args:
            shape (tuple[int, int], optional): The input and output size of the tree. Defaults to (1, 1).
            min_depth (int, optional): The minimum depth of the tree (ie: the starting height of a tree). Defaults to 3.
            max_size (int, optional): The maximum size of the tree (ie: the maximum number of nodes). Defaults to 30.
            vertex (Op | list[Op] | None, optional): Operations to use for internal nodes. Can be a single Op or a list of Ops. Defaults to None.
            leaf (Op | list[Op] | None, optional): Operations to use for leaf nodes. Can be a single Op or a list of Ops. Defaults to None.
            root (Op | list[Op] | None, optional): Operations to use for the root node. Can be a single Op or a list of Ops. Defaults to None.

        Raises:
            ValueError: If input_size or output_size is less than 1.
            ValueError: If min_depth is less than 1.
            ValueError: If max_size is less than 1.
        """
        input_size, output_size = shape

        if input_size < 1 or output_size < 1:
            raise ValueError("Input and output size must be at least 1")
        if min_depth < 1:
            raise ValueError("Minimum depth must be at least 1")
        if max_size < 1:
            raise ValueError("Maximum size must be at least 1")

        ops_map = OpBuilder(
            dtype=dtype,
            leaf=[Op.var(i, dtype=dtype) for i in range(input_size)],
            vertex=vertex if vertex is not None else Op.default_vertex_ops(dtype),
            root=root if root is not None else Op.linear(dtype),
        )

        # ops_config = OpsConfig(
        #     vertex=vertex, leaf=leaf, root=root, values=values
        # ).build_ops_map(input_size=input_size, fill_invalid=True)

        self._pyobj = PyTreeCodec(
            output_size,
            min_depth,
            max_size,
            ops={
                key: [op.__backend__() for op in ops]
                for key, ops in ops_map.ops_map.items()
            },
        )

    def encode(self) -> Genotype[Op]:
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype) -> Tree:
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return Tree.from_rust(
            self.__backend__().decode_py(genotype=genotype.__backend__())
        )
