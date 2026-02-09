from __future__ import annotations

from abc import ABC, abstractmethod

from radiate.genome import Genotype, GeneType


class CodecBase[T, D](ABC):
    """Base for codecs. Subclasses must set gene_type and implement encode/decode."""

    gene_type: GeneType

    @abstractmethod
    def encode(self) -> Genotype[T]:
        """
        Encodes the codec into a Genotype.
        :return: A Genotype instance.
        """
        pass

    @abstractmethod
    def decode(self, genotype: Genotype[T]) -> D:
        """
        Decodes a Genotype into its representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded representation of the Genotype.
        """
        pass
