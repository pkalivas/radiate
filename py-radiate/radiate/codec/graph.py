from typing import List, Optional, Dict, TypeAlias, Union, Tuple
from .codec import CodecBase
from ..gp import Op, Graph
from radiate.genome import Genotype
from radiate.radiate import PyGraphCodec

NodeValues: TypeAlias = Union[List[Op], Op]


class GraphCodec(CodecBase):
    def __init__(self, codec: PyGraphCodec):
        self.codec = codec

    def encode(self) -> "Genotype":
        return Genotype(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> "Graph":
        if genotype.gene_type() != "GraphNode":
            raise ValueError("genotype must be of type 'graph'.")
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return Graph(self.codec.decode_py(genotype.py_genotype()))

    @staticmethod
    def weighted_directed(
        shape: Tuple[int, int],
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]] | List[Tuple[str, List[Op]]]] = None,
    ) -> "GraphCodec":
        return GraphCodec.__build_common(
            name="weighted_directed",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
        )

    @staticmethod
    def weighted_recurrent(
        shape: Tuple[int, int],
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]] | List[Tuple[str, List[Op]]]] = None,
    ) -> "GraphCodec":
        return GraphCodec.__build_common(
            name="weighted_recurrent",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
        )

    @staticmethod
    def directed(
        shape: Tuple[int, int],
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]] | List[Tuple[str, List[Op]]]] = None,
    ) -> "GraphCodec":
        return GraphCodec.__build_common(
            name="directed",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
        )

    @staticmethod
    def recurrent(
        shape: Tuple[int, int],
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]]] = None,
    ) -> "GraphCodec":
        return GraphCodec.__build_common(
            name="recurrent",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
        )

    @staticmethod
    def __build_common(
        name: str = "directed",
        shape: Tuple[int, int] = (1, 1),
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]]] = None,
    ) -> "GraphCodec":
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

        if name == "weighted_directed":
            return GraphCodec(
                PyGraphCodec("weighted_directed", input_size, output_size, ops_map)
            )
        elif name == "weighted_recurrent":
            return GraphCodec(
                PyGraphCodec("weighted_recurrent", input_size, output_size, ops_map)
            )
        elif name == "recurrent":
            return GraphCodec(
                PyGraphCodec("recurrent", input_size, output_size, ops_map)
            )
        else:
            if name != "directed":
                raise ValueError(f"Unknown graph type: {name}")
        # Default to directed graph
        return GraphCodec(PyGraphCodec("directed", input_size, output_size, ops_map))
