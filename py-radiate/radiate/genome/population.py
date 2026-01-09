from __future__ import annotations

from typing import TYPE_CHECKING

from typing import Iterable
from radiate.radiate import PyPopulation
from .phenotype import Phenotype
from ..wrapper import PyObject

if TYPE_CHECKING:
    from radiate.genome.gene import GeneType


class Population[T](PyObject[PyPopulation]):
    """
    Represents a population in a genetic algorithm.
    """

    def __init__(self, individuals: Iterable[Phenotype[T]]):
        super().__init__()

        if isinstance(individuals, Iterable):
            self._pyobj = PyPopulation(
                list(map(lambda p: p.__backend__(), individuals))
            )

    def __repr__(self):
        return self._pyobj.__repr__()

    def __len__(self):
        """
        Returns the number of individuals in the population.
        :return: Number of individuals in the population.
        """
        return len(self._pyobj)

    def __iter__(self) -> Iterable[Phenotype[T]]:
        """
        Returns an iterator over the individuals in the population.
        :return: An iterator over the individuals in the population.
        """
        for phenotype in self.phenotypes():
            yield phenotype

    def __getitem__(self, index: int) -> Phenotype[T]:
        """
        Returns the Phenotype at the specified index.
        :param index: The index of the Phenotype to retrieve.
        :return: The Phenotype at the specified index.
        """
        return self.phenotypes()[index]

    def gene_type(self) -> "GeneType":
        """
        Returns the type of the genes in the population.
        :return: The gene type as a string.
        """
        from . import GeneType

        return GeneType.from_str(self._pyobj.gene_type())

    def phenotypes(self) -> list[Phenotype[T]]:
        """
        Get the phenotypes of the population.
        :return: The phenotypes of the population.
        """
        return self.try_get_cache(
            "phenotypes_cache",
            lambda: [Phenotype.from_rust(p) for p in self.__backend__().phenotypes],
        )
