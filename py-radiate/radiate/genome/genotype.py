from __future__ import annotations

from typing import List
from radiate.radiate import PyGenotype
from .chromosome import Chromosome


class Genotype:
    """
    Represents a genotype in a genome.
    """

    def __init__(
        self,
        genotype: PyGenotype | None = None,
        chromosomes: List[Chromosome] | Chromosome | None = None,
    ):
        """
        Initializes a Genotype instance.

        :param chromosomes: A list of Chromosome instances.
        """
        if genotype is not None:
            if isinstance(genotype, PyGenotype):
                self.__inner = genotype
            else:
                raise TypeError("genotype must be an instance of PyGenotype")
        elif chromosomes is not None:
            if isinstance(chromosomes, Chromosome):
                chromosomes = [chromosomes]
            if isinstance(chromosomes, list):
                if all(isinstance(chromo, Chromosome) for chromo in chromosomes):
                    self.__inner = PyGenotype(
                        [chromo.py_chromosome() for chromo in chromosomes]
                    )
                else:
                    raise ValueError("All chromosomes must be instances of Chromosome")
            else:
                raise TypeError(
                    "chromosomes must be a Chromosome instance or a list of Chromosome instances"
                )
        else:
            raise ValueError("Either genotype or chromosomes must be provided")

    def __repr__(self):
        return f"Genotype({self.__inner.chromosomes})"

    def __len__(self):
        """
        Returns the length of the genotype.
        :return: Length of the genotype.
        """
        return len(self.__inner.chromosomes)

    def __eq__(self, value):
        if not isinstance(value, Genotype):
            return False
        return self.__inner == value.__inner

    def __getitem__(self, index: int) -> Chromosome:
        """
        Returns the chromosome at the specified index.
        :param index: Index of the chromosome to retrieve.
        :return: Chromosome instance at the specified index.
        """
        if not isinstance(index, int):
            raise TypeError("Index must be an integer")
        if index < 0 or index >= len(self.__inner.chromosomes):
            raise IndexError("Index out of range")
        return Chromosome(chromosome=self.__inner.chromosomes[index])

    def __iter__(self):
        """
        Returns an iterator over the chromosomes in the genotype.
        :return: An iterator over the chromosomes in the genotype.
        """
        for chromosome in self.__inner.chromosomes:
            yield Chromosome(chromosome=chromosome)

    def py_genotype(self) -> PyGenotype:
        """
        Returns the underlying PyGenotype instance.
        :return: The PyGenotype instance associated with this Genotype.
        """
        return self.__inner

    def gene_type(self) -> str:
        """
        Returns the type of the genes in the genotype.
        :return: The gene type as a string.
        """
        return self.__inner.gene_type().__repr__()

    def chromosomes(self) -> List[Chromosome]:
        """
        Returns the chromosomes of the genotype.
        :return: A list of Chromosome instances.
        """
        return [
            Chromosome(chromosome=chromosome) for chromosome in self.__inner.chromosomes
        ]
