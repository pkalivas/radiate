from typing import List
from .codec import CodecBase
from radiate.radiate import PyCharCodec


class CharCodec(CodecBase):
    def __init__(self, codec: PyCharCodec):
        if not isinstance(codec, PyCharCodec):
            raise TypeError("codec must be an instance of PyCharCodec.")
        self.codec = codec

    @staticmethod
    def matrix(chromosomes: List[int], char_set: str | List[str] = None) -> 'CharCodec':
        """
        Initialize the char codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        :param value_range: Minimum and maximum value for the genes.
        """

        if isinstance(char_set, str):
            char_set = list(char_set)

        if char_set is not None:
            for char in char_set:
                if not isinstance(char, str) or len(char) != 1:
                    raise ValueError(
                        "Character set must be a string or list of single-character strings."
                    )

        return CharCodec(PyCharCodec.matrix(chromosomes, char_set))

    @staticmethod
    def vector(length: int, char_set: str | List[str] = None) -> 'CharCodec':
        """
        Initialize the char codec with a single chromosome of specified length.
        :param length: Length of the chromosome.
        :param char_set: Character set to use for encoding.
        """
        return CharCodec(PyCharCodec.vector(length, char_set))
