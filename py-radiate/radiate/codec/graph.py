from __future__ import annotations

from radiate.radiate import PyGraphCodec

from .._bridge.wrapper import RsObject
from .._typing import AtLeastOne
from ..genome import GeneType, Genotype
from ..gp import Graph, Op, OpsConfig
from .base import CodecBase


class GraphCodec(CodecBase[Op, Graph], RsObject):
    gene_type = GeneType.GRAPH

    def __init__(
        self,
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
        graph_type: str = "directed",
    ):
        input_size, output_size = shape
        if input_size < 1 or output_size < 1:
            raise ValueError("Input and output size must be at least 1")

        config = OpsConfig(
            vertex=vertex, edge=edge, output=output, values=values
        ).build_ops_map(input_size, fill_invalid=True)

        if graph_type not in [
            "weighted_directed",
            "weighted_recurrent",
            "recurrent",
            "directed",
            "gru",
            "lstm",
        ]:
            raise ValueError(f"Unknown graph type: {graph_type}")

        self._pyobj = PyGraphCodec(
            graph_type=graph_type,
            input_size=shape[0],
            output_size=shape[1],
            ops=config,
            max_nodes=max_nodes,
        )

    def encode(self) -> Genotype[Op]:
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype) -> Graph:
        if genotype.gene_type() != GeneType.GRAPH:
            raise ValueError(f"genotype must be of type {genotype.gene_type()}.")
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return Graph.from_rust(self.__backend__().decode_py(genotype.__backend__()))

    @staticmethod
    def weighted_directed(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            values,
            max_nodes,
            graph_type="weighted_directed",
        )

    @staticmethod
    def weighted_recurrent(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec(
            shape,
            vertex,
            edge,
            output,
            values,
            max_nodes,
            graph_type="weighted_recurrent",
        )

    @staticmethod
    def directed(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec(
            shape, vertex, edge, output, values, max_nodes, graph_type="directed"
        )

    @staticmethod
    def recurrent(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec(
            shape, vertex, edge, output, values, max_nodes, graph_type="recurrent"
        )

    @staticmethod
    def gru(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec(
            shape, vertex, edge, output, values, max_nodes, graph_type="gru"
        )

    @staticmethod
    def lstm(
        shape: tuple[int, int],
        vertex: AtLeastOne[Op] | None = None,
        edge: AtLeastOne[Op] | None = None,
        output: AtLeastOne[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
    ) -> GraphCodec:
        return GraphCodec(
            shape, vertex, edge, output, values, max_nodes, graph_type="lstm"
        )
