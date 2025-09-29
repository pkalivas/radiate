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
        for phenotype in self._pyobj.phenotypes:
            yield Phenotype.from_rust(phenotype)

    def __getitem__(self, index: int) -> Phenotype[T]:
        """
        Returns the Phenotype at the specified index.
        :param index: The index of the Phenotype to retrieve.
        :return: The Phenotype at the specified index.
        """
        return Phenotype.from_rust(self._pyobj[index])

    def __setitem__(self, index: int, value: Phenotype[T]):
        """
        Sets the Phenotype at the specified index.
        :param index: The index of the Phenotype to set.
        :param value: The Phenotype to set at the specified index.
        """
        if not isinstance(value, Phenotype):
            raise TypeError("Value must be an instance of Phenotype")
        self._pyobj[index] = value.__backend__()

    def gene_type(self) -> "GeneType":
        """
        Returns the type of the genes in the population.
        :return: The gene type as a string.
        """
        from . import GeneType

        return GeneType.from_str(self._pyobj.gene_type())
