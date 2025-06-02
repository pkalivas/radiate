from typing import List, Any
from .codec import CodecBase
from ..genome import Chromosome

from radiate.radiate import PyBitCodec

class BitCodec(CodecBase):
    """
    BitCodec is a class that represents a codec for bit-based chromosomes.
    It is used to encode and decode chromosomes into bit strings.
    """

    def __init__(self, chromosomes: List[int]):
        """
        Initialize the bit codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        """
        self.codec = PyBitCodec(
            chromosome_lengths=chromosomes,
        )

    def encode(self) -> List[Any]:
        """
        Encode the chromosomes into a list of Chromosome objects.
        :return: A list of Chromosome objects.
        """
        return [Chromosome(chromosome=chromosome) for chromosome in self.codec.py_encode().chromosomes]
