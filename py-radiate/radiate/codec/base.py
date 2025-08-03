from abc import ABC, abstractmethod

from radiate.genome import Genotype


class CodecBase[T, D](ABC):
    @abstractmethod
    def encode(self) -> Genotype[T]:
        """
        Encodes the codec into a Genotype.
        :return: A Genotype instance.
        """
        pass

    @abstractmethod
    def decode(self, genotype: Genotype) -> D:
        """
        Decodes a Genotype into its representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded representation of the Genotype.
        """
        pass
