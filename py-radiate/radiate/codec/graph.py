from __future__ import annotations

from enum import StrEnum

from radiate.gp.op import OpBuilder
from radiate.radiate import PyGraphCodec

from .._bridge import RsObject
from .._typing import AtLeastOne
from ..dsl.dtype import DataType, DataTypeClass, Float32, Float64
from ..genome import GeneType, Genotype
from ..gp import Graph, Op
from .base import CodecBase


class GraphType(StrEnum):
    WEIGHTED_DIRECTED = "weighted_directed"
    WEIGHTED_RECURRENT = "weighted_recurrent"
    RECURRENT = "recurrent"
    DIRECTED = "directed"
    GRU = "gru"
    LSTM = "lstm"


class GraphCodec(CodecBase[Op, Graph], RsObject):
    def __init__(
        self,
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        max_nodes: int | None = None,
        graph_type: GraphType = GraphType.DIRECTED,
        dtype: DataTypeClass | DataType = Float64,
    ):
        input_size, output_size = shape
        if input_size < 1 or output_size < 1:
            raise ValueError("Input and output size must be at least 1")

        ops_map = OpBuilder(
            dtype=dtype,
            input=[Op.var(i, dtype=dtype) for i in range(input_size)],
            vertex=vertex if vertex is not None else Op.default_vertex_ops(dtype),
            edge=edge if edge is not None else Op.default_edge_ops(dtype),
            output=output if output is not None else Op.linear(dtype),
        )

        if dtype not in [Float32, Float64]:
            raise TypeError(f"GraphCodec only supports Float32 & Float64, got {dtype}.")

        self._pyobj = PyGraphCodec(
            graph_type=graph_type,
            input_size=shape[0],
            output_size=shape[1],
            ops={
                key: [op.__backend__() for op in ops]
                for key, ops in ops_map.ops.items()
            },
            max_nodes=max_nodes,
            dtype=str(dtype),
        )

    def encode(self) -> Genotype[Op]:
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype) -> Graph:
        if genotype.gene_type() != GeneType.GRAPH:
            raise ValueError(f"genotype must be of type {genotype.gene_type()}.")
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return Graph.from_rust(self.__backend__().decode_py(genotype.__backend__()))

    @property
    def gene_type(self) -> GeneType:
        return GeneType.GRAPH

    @staticmethod
    def weighted_directed(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        max_nodes: int | None = None,
        dtype: DataTypeClass | DataType = Float64,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            max_nodes,
            graph_type=GraphType.WEIGHTED_DIRECTED,
            dtype=dtype,
        )

    @staticmethod
    def weighted_recurrent(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        max_nodes: int | None = None,
        dtype: DataTypeClass | DataType = Float64,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            max_nodes,
            graph_type=GraphType.WEIGHTED_RECURRENT,
            dtype=dtype,
        )

    @staticmethod
    def directed(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        max_nodes: int | None = None,
        dtype: DataTypeClass | DataType = Float64,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            max_nodes,
            graph_type=GraphType.DIRECTED,
            dtype=dtype,
        )

    @staticmethod
    def recurrent(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        max_nodes: int | None = None,
        dtype: DataTypeClass | DataType = Float64,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            max_nodes,
            graph_type=GraphType.RECURRENT,
            dtype=dtype,
        )

    @staticmethod
    def gru(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        max_nodes: int | None = None,
        dtype: DataTypeClass | DataType = Float64,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            max_nodes,
            graph_type=GraphType.GRU,
            dtype=dtype,
        )

    @staticmethod
    def lstm(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        max_nodes: int | None = None,
        dtype: DataTypeClass | DataType = Float64,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            max_nodes,
            graph_type=GraphType.LSTM,
            dtype=dtype,
        )
