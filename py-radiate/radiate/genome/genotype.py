from __future__ import annotations

from collections.abc import Iterable
from radiate.genome.gene import GeneType
from radiate.genome.wrapper import PythonWrapper
from radiate.radiate import PyGenotype
from .chromosome import Chromosome


class Genotype[T](PythonWrapper[PyGenotype]):
    """
    Represents a genotype in a genome.
    """

    def __init__(
        self,
        chromosomes: Iterable[Chromosome[T]] | Chromosome[T] | None = None,
    ):
        """
        Initializes a Genotype instance.

        :param chromosomes: A list of Chromosome instances.
        """
        super().__init__()

        if isinstance(chromosomes, Chromosome):
            self._pyobj = PyGenotype([chromosomes.to_python()])
        elif isinstance(chromosomes, Iterable):
            if all(isinstance(chromo, Chromosome) for chromo in chromosomes):
                self._pyobj = PyGenotype(list(map(lambda c: c.to_python(), chromosomes)))
            else:
                raise ValueError("All chromosomes must be instances of Chromosome")

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
        if not isinstance(index, int):
            raise TypeError("Index must be an integer")
        if index < 0 or index >= len(self._pyobj):
            raise IndexError("Index out of range")
        return self._pyobj[index]

    def __iter__(self):
        """
        Returns an iterator over the chromosomes in the genotype.
        :return: An iterator over the chromosomes in the genotype.
        """
        for chromosome in self._pyobj.chromosomes:
            yield Chromosome.from_python(chromosome)

    def gene_type(self) -> GeneType:
        """
        Returns the type of the genes in the genotype.
        :return: The gene type as a string.
        """
        return GeneType.from_str(self._pyobj.gene_type())
