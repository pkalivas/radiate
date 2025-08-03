from __future__ import annotations

from typing import List

from .base import CodecBase

from radiate.radiate import PyPermutationCodec
from radiate.genome import Genotype


class PermutationCodec[T](CodecBase[T, List[T]]):
    def __init__(self, alleles: List[T]):
        """
        Initialize the permutation codec with a PyPermutationCodec instance.
        :param codec: An instance of PyPermutationCodec.
        """
        if not isinstance(alleles, list):
            raise TypeError("alleles must be a list of elements.")
        self.alleles = alleles
        self.codec = PyPermutationCodec(alleles)

    def encode(self) -> Genotype[T]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_python(self.codec.encode_py())

    def decode(self, genotype: Genotype[T]) -> List[T]:
        """
        Decode a Genotype into its permutation representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded permutation representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype=genotype.to_python())
