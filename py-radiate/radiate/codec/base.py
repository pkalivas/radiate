from __future__ import annotations

from typing import TYPE_CHECKING
from abc import ABC, abstractmethod

from radiate._bridge.wrapper import RsObject


if TYPE_CHECKING:
    from radiate.genome import Genotype, GeneType, Population


class CodecBase[T, D](RsObject, ABC):
    gene_type: "GeneType"

    @abstractmethod
    def encode(self) -> "Genotype[T]":
        raise NotImplementedError("encode method must be implemented by subclasses.")

    @abstractmethod
    def decode(self, genotype: "Genotype[T]") -> D:
        raise NotImplementedError("decode method must be implemented by subclasses.")

    def population(self, size: int = 100) -> "Population[T]":
        """
        Generate a population of genotypes using the codec's encoding method. This is pretty much just a helper function.

        :param size: The number of genotypes to generate.
        :return: A population of genotypes.
        """
        from radiate.genome import Population, Phenotype

        phenotypes = [Phenotype(genotype=self.encode()) for _ in range(size)]
        return Population(phenotypes)
