from __future__ import annotations

from radiate._typing import NodeValues

from .base import CodecBase
from ..gp import Op, Graph
from radiate.genome import Genotype, GeneType
from radiate.radiate import PyGraphCodec


class GraphCodec(CodecBase[Op, Graph]):
    def __init__(self, codec: PyGraphCodec):
        self.codec = codec

    def encode(self) -> Genotype[Op]:
        return Genotype.from_rust(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> Graph:
        if genotype.gene_type() != GeneType.GRAPH:
            raise ValueError(f"genotype must be of type {genotype.gene_type()}.")
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return Graph(self.codec.decode_py(genotype.__backend__()))

    @staticmethod
    def weighted_directed(
        shape: tuple[int, int],
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec.__build_common(
            name="weighted_directed",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

    @staticmethod
    def weighted_recurrent(
        shape: tuple[int, int],
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec.__build_common(
            name="weighted_recurrent",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

    @staticmethod
    def directed(
        shape: tuple[int, int],
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec.__build_common(
            name="directed",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

    @staticmethod
    def recurrent(
        shape: tuple[int, int],
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec.__build_common(
            name="recurrent",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

    @staticmethod
    def gru(
        shape: tuple[int, int],
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec.__build_common(
            name="gru",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

    @staticmethod
    def lstm(
        shape: tuple[int, int],
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec.__build_common(
            name="lstm",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

    @staticmethod
    def __build_common(
        name: str = "directed",
        shape: tuple[int, int] = (1, 1),
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        input_size, output_size = shape

        if input_size < 1 or output_size < 1:
            raise ValueError("Input and output size must be at least 1")

        ops_map = {"input": [Op.var(i) for i in range(input_size)]}
        if values is not None:
            if isinstance(values, list):
                values = dict(values)
            ops_map = values | ops_map
        else:
            if vertex is not None:
                ops_map["vertex"] = [vertex] if isinstance(vertex, Op) else vertex
            if edge is not None:
                ops_map["edge"] = [edge] if isinstance(edge, Op) else edge
            if output is not None:
                ops_map["output"] = [output] if isinstance(output, Op) else output

        if name not in [
            "weighted_directed",
            "weighted_recurrent",
            "recurrent",
            "directed",
            "gru",
            "lstm",
        ]:
            raise ValueError(f"Unknown graph type: {name}")

        return GraphCodec(
            PyGraphCodec(
                name,
                input_size,
                output_size,
                ops_map,
                max_nodes,
            )
        )
