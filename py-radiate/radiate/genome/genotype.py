from __future__ import annotations

from typing import TYPE_CHECKING

from collections.abc import Iterable
from radiate.wrapper import PyObject
from radiate.radiate import PyGenotype
from .chromosome import Chromosome

if TYPE_CHECKING:
    from radiate.genome import GeneType


class Genotype[T](PyObject[PyGenotype]):
    """
    Represents a genotype in a genome.
    """

    def __init__(
        self,
        chromosomes: Iterable[Chromosome[T]] | Chromosome[T] | None = None,
    ):
        super().__init__()

        if isinstance(chromosomes, Chromosome):
            self._pyobj = PyGenotype([chromosomes.__backend__()])
        elif isinstance(chromosomes, Iterable):
            self._pyobj = PyGenotype(list(map(lambda c: c.__backend__(), chromosomes)))

    def __repr__(self):
        return self._pyobj.__repr__()

    def __len__(self):
        """
        Returns the length of the genotype.
        :return: Length of the genotype.
        """
        return len(self._pyobj)

    def __getitem__(self, index: int) -> Chromosome[T]:
        """
        Returns the chromosome at the specified index.
        :param index: Index of the chromosome to retrieve.
        :return: Chromosome instance at the specified index.
        """
        return Chromosome.from_rust(self._pyobj[index])

    def __iter__(self):
        """
        Returns an iterator over the chromosomes in the genotype.
        :return: An iterator over the chromosomes in the genotype.
        """
        for chromosome in self._pyobj.chromosomes:
            yield Chromosome.from_rust(chromosome)

    def gene_type(self) -> "GeneType":
        """
        Returns the type of the genes in the genotype.
        :return: The gene type as a string.
        """
        from . import GeneType

        return GeneType.from_str(self._pyobj.gene_type())
