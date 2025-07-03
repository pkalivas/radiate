from .codec import CodecBase
from radiate.radiate import PyPermutationCodec
from radiate.genome import Genotype
from typing import List, Any


class PermutationCodec(CodecBase):
    def __init__(self, alleles: List[Any]):
        """
        Initialize the permutation codec with a PyPermutationCodec instance.
        :param codec: An instance of PyPermutationCodec.
        """
        super().__init__()
        if not isinstance(alleles, list):
            raise TypeError("alleles must be a list of elements.")
        self.alleles = alleles
        self.codec = PyPermutationCodec(alleles)

    def encode(self) -> Genotype:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> Any:
        """
        Decode a Genotype into its permutation representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded permutation representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype.py_genotype())
