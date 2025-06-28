from typing import List, Optional
from .codec import CodecBase
from ..gp import Op
from radiate.genome import Genotype

from radiate.radiate import PyGraphCodec


class GraphCodec(CodecBase):
    def __init__(self, codec: PyGraphCodec):
        self.codec = codec

    def encode(self) -> "Genotype":
        return Genotype(self.codec.encode_py())

    @staticmethod
    def directed(
        input_size: int,
        output_size: int,
        vertex: Optional[List[Op]] = None,
        edge: Optional[List[Op]] = None,
        output: Optional[List[Op]] = None,
    ) -> "GraphCodec":
        inputs = [Op.var(i) for i in range(input_size)]
        ops_map = {"input": inputs, "vertex": vertex, "edge": edge, "output": output}
        return GraphCodec(PyGraphCodec.directed(input_size, output_size, ops_map))
