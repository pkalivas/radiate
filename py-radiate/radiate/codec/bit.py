from __future__ import annotations
from typing import List, Any
from .codec import CodecBase
from radiate.radiate import PyBitCodec
from radiate.genome import Genotype


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

    def encode(self) -> Genotype:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> Any:
        """
        Decode a Genotype into its bit representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded bit representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype.py_genotype())

    @staticmethod
    def matrix(chromosome_lengths: List[int]) -> "BitCodec":
        """
        Initialize the bit codec with a matrix of chromosomes.
        Args:
            chromosome_lengths: A list of integers specifying the lengths of each chromosome.
        Returns:
            A new BitCodec instance with matrix configuration.

        Example
        --------
        >>> rd.BitCodec.matrix(chromosome_lengths=[5, 5])
        BitCodec(...)
        """
        return BitCodec(PyBitCodec.matrix(chromosome_lengths=chromosome_lengths))

    @staticmethod
    def vector(length: int) -> "BitCodec":
        """
        Initialize the bit codec with a single chromosome of specified length.
        Args:
            length: Length of the chromosome.
        Returns:
            A new BitCodec instance with vector configuration.

        Example
        --------
        >>> rd.BitCodec.vector(length=5)
        BitCodec(...)
        """
        return BitCodec(PyBitCodec.vector(length=length))
