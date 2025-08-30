from __future__ import annotations


from .base import CodecBase

from radiate.radiate import PyBitCodec
from radiate.genome import Genotype


class BitCodec[T](CodecBase[bool, T]):
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

    def encode(self) -> Genotype[bool]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.codec.encode_py())

    def decode(self, genotype: Genotype[bool]) -> T:
        """
        Decode a Genotype into its bit representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded bit representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype.__backend__())

    @staticmethod
    def matrix(
        shape: list[int] | tuple[int, int], use_numpy: bool = False
    ) -> BitCodec[list[list[bool]]]:
        """
        Initialize the bit codec with a matrix of chromosomes.
        Args:
            shape: A list of integers specifying the shape of the matrix.
        Returns:
            A new BitCodec instance with matrix configuration.

        Example
        --------
        >>> rd.BitCodec.matrix(chromosome_lengths=[5, 5])
        BitCodec(...)
        """
        if isinstance(shape, tuple):
            if len(shape) != 2:
                raise ValueError("Shape must be a tuple of (rows, cols).")
            rows, cols = shape
            if rows < 1 or cols < 1:
                raise ValueError("Rows and columns must be at least 1.")
            shape = [cols for _ in range(rows)]
        elif isinstance(shape, list):
            if not all(isinstance(x, int) and x > 0 for x in shape):
                raise ValueError("Shape must be a list of positive integers.")

        return BitCodec(
            PyBitCodec.matrix(chromosome_lengths=shape, use_numpy=use_numpy)
        )

    @staticmethod
    def vector(length: int = 8, use_numpy: bool = False) -> BitCodec[list[bool]]:
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
        return BitCodec(PyBitCodec.vector(length, use_numpy=use_numpy))
