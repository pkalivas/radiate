from typing import Any, Callable

from radiate.genome.genotype import Genotype
from . import CodecBase
from radiate.radiate import PyAnyCodec

class AnyCodec(CodecBase):
    def __init__(self, encoder: Callable[[], Any]):
        """
        Initialize the AnyCodec with encoder and decoder functions.
        :param encoder: A callable that encodes a value.
        :param decoder: A callable that decodes a value.
        """
        self.codec = PyAnyCodec(encoder)

    def encode(self) -> Any:
        """
        Encodes the codec into a PyAnyCodec.
        :return: A PyAnyCodec instance.
        """
        return self.codec.encode_py()
    
    def decode(self, genotype: Genotype) -> Any:
        """
        Decodes a PyAnyCodec into its representation.
        :param genotype: A PyAnyCodec instance to decode.
        :return: The decoded representation of the PyAnyCodec.
        """
        return self.codec.decode_py(genotype)