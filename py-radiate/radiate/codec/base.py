import abc

from typing import Any
from radiate.genome import Genotype


class CodecBase(abc.ABC):
    def encode(self) -> Genotype:
        """
        Encodes the codec into a Genotype.
        :return: A Genotype instance.
        """
        raise NotImplementedError("Subclasses must implement this method.")

    def decode(self, genotype: Genotype) -> Any:
        """
        Decodes a Genotype into its representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded representation of the Genotype.
        """
        raise NotImplementedError("Subclasses must implement this method.")
