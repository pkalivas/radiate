from typing import List, Optional, Dict, TypeAlias, Union, Tuple
from .codec import CodecBase
from ..gp import Op
from radiate.genome import Genotype
from radiate.radiate import PyGraphCodec

NodeValues: TypeAlias = Union[List[Op], Op]


class GraphCodec(CodecBase):
    def __init__(self, codec: PyGraphCodec):
        self.codec = codec

    def encode(self) -> "Genotype":
        return Genotype(self.codec.encode_py())

    @staticmethod
    def directed(
        shape: Tuple[int, int],
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]] | List[Tuple[str, List[Op]]]] = None,
    ) -> "GraphCodec":
        return GraphCodec.__build_common(
            recurrent=False,
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
            recurrent=True,
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
        )

    @staticmethod
    def __build_common(
        recurrent: bool,
        shape: Tuple[int, int],
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

        if recurrent:
            return GraphCodec(PyGraphCodec("recurrent", input_size, output_size, ops_map))
        return GraphCodec(PyGraphCodec("directed", input_size, output_size, ops_map))
