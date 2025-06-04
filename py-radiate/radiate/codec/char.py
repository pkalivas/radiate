from typing import List

from .codec import CodecBase
from radiate.radiate import PyCharCodec

class CharCodec(CodecBase):
    def __init__(self, chromosomes: List[int], char_set: str | List[str] = None):
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

        self.codec = PyCharCodec(
            chromosome_lengths=chromosomes,
            char_set="".join(set(char_set)) if char_set else None,
        )