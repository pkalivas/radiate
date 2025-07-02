from __future__ import annotations
from typing import List, Any, Tuple
from .codec import CodecBase
from radiate.radiate import PyCharCodec
from radiate.genome import Genotype


class CharCodec(CodecBase):
    def __init__(self, codec: PyCharCodec):
        if not isinstance(codec, PyCharCodec):
            raise TypeError("codec must be an instance of PyCharCodec.")
        self.codec = codec

    def encode(self) -> Genotype:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> Any:
        """
        Decode a Genotype into its character representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded character representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype.py_genotype())

    @staticmethod
    def matrix(
        chromosomes: List[int] | Tuple[int, int],
        char_set: str | List[str] = None,
        use_numpy: bool = False,
    ) -> "CharCodec":
        """
        Initialize the char codec with number of chromosomes and value bounds.
        Args:
            chromosomes: A list of integers specifying the lengths of each chromosome.
            char_set: A string or list of strings representing the character set.
        Returns:
            A new CharCodec instance with matrix configuration.

        Example
        --------
        >>> rd.CharCodec.matrix(chromosomes=[5, 5], char_set="01")
        CharCodec(...)
        """

        if isinstance(chromosomes, tuple):
            if len(chromosomes) != 2:
                raise ValueError("Chromosomes must be a tuple of (rows, cols).")
            rows, cols = chromosomes
            if rows < 1 or cols < 1:
                raise ValueError("Rows and columns must be at least 1.")
            chromosomes = [cols for _ in range(rows)]
        if isinstance(chromosomes, list):
            if not all(isinstance(x, int) and x > 0 for x in chromosomes):
                raise ValueError("Chromosomes must be a list of positive integers.")

        if char_set is not None:
            for char in char_set:
                if not isinstance(char, str) or len(char) != 1:
                    raise ValueError(
                        "Character set must be a string or list of single-character strings."
                    )

        return CharCodec(PyCharCodec.matrix(chromosomes, char_set, use_numpy=use_numpy))

    @staticmethod
    def vector(
        length: int, char_set: str | List[str] = None, use_numpy: bool = False
    ) -> "CharCodec":
        """
        Initialize the char codec with a single chromosome of specified length.
        Args:
            length: Length of the chromosome.
            char_set: Character set to use for encoding.
        Returns:
            A new CharCodec instance with vector configuration.

        Example
        --------
        >>> rd.CharCodec.vector(length=5, char_set="01")
        CharCodec(...)
        """
        return CharCodec(PyCharCodec.vector(length, char_set, use_numpy=use_numpy))
