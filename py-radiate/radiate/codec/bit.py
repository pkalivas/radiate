from typing import List
from .codec import CodecBase
from radiate.radiate import PyBitCodec


class BitCodec(CodecBase):
    """
    BitCodec is a class that represents a codec for bit-based chromosomes.
    It is used to encode and decode chromosomes into bit strings.
    """

    def __init__(self, codec: PyBitCodec):
        """
        Initialize the bit codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        """
        if not isinstance(codec, PyBitCodec):
            raise TypeError("codec must be an instance of PyBitCodec.")
        self.codec = codec

    @staticmethod
    def matrix(chromosome_lengths: List[int]):
        """
        Initialize the bit codec with a matrix of chromosomes.
        :param chromosome_lengths: List of integers representing the lengths of each chromosome.
        """
        return BitCodec(PyBitCodec.matrix(chromosome_lengths=chromosome_lengths))

    @staticmethod
    def vector(length: int):
        """
        Initialize the bit codec with a single chromosome of specified length.
        :param length: Length of the chromosome.
        """
        return BitCodec(PyBitCodec.vector(length=length))
