from typing import List, Optional, Dict, TypeAlias, Union
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
        input_size: int,
        output_size: int,
        vertex: Optional[NodeValues] = None,
        edge: Optional[NodeValues] = None,
        output: Optional[NodeValues] = None,
        values: Optional[Dict[str, List[Op]]] = None,
    ) -> "GraphCodec":
        inputs = [Op.var(i) for i in range(input_size)]
        if values is not None:
            ops_map = values | {"input": inputs}
        else:
            if isinstance(vertex, Op):
                vertex = [vertex]                
            if isinstance(edge, Op):
                edge = [edge]
            if isinstance(output, Op):
                output = [output]
                            
            ops_map = {"input": inputs, "vertex": vertex, "edge": edge, "output": output}
        return GraphCodec(PyGraphCodec.directed(input_size, output_size, ops_map))
