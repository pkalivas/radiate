from __future__ import annotations

from .base import CodecBase

from radiate.radiate import PyPermutationCodec
from radiate.genome import Genotype, GeneType
from radiate._bridge.wrapper import RsObject


class PermutationCodec[T](CodecBase[T, list[T]], RsObject):
    gene_type = GeneType.PERMUTATION

    def __init__(self, alleles: list[T]):
        """
        Initialize the permutation codec with a PyPermutationCodec instance.
        :param codec: An instance of PyPermutationCodec.
        """
        self.alleles = alleles
        self._pyobj = PyPermutationCodec(alleles)

    def encode(self) -> Genotype[T]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self._pyobj.encode_py())

    def decode(self, genotype: Genotype[T]) -> list[T]:
        """
        Decode a Genotype into its permutation representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded permutation representation of the Genotype.
        """
        return self._pyobj.decode_py(genotype=genotype.__backend__())
