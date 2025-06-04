from typing import List
from .codec import CodecBase

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
