from __future__ import annotations

from typing import Iterable, List
from radiate.genome.gene import GeneType
from radiate.radiate import PyPopulation
from .phenotype import Phenotype
from .wrapper import PythonWrapper


class Population[T](PythonWrapper[PyPopulation]):
    """
    Represents a population in a genetic algorithm.
    """

    def __init__(self, individuals: Iterable[Phenotype[T]]):
        """
        Initializes a Population instance.

        :param individuals: A list of Phenotype instances.
        """
        super().__init__()

        if isinstance(individuals, Iterable):
            self._pyobj = PyPopulation(
                [phenotype.to_python() for phenotype in individuals]
            )
        elif isinstance(individuals, List):
            if all(isinstance(ind, Phenotype) for ind in individuals):
                self._pyobj = PyPopulation(
                    [phenotype.to_python() for phenotype in individuals]
                )
            else:
                raise ValueError(
                    "All individuals must be instances of Phenotype or Genotype"
                )

    def __repr__(self):
        return self._pyobj.__repr__()

    def __len__(self):
        """
        Returns the number of individuals in the population.
        :return: Number of individuals in the population.
        """
        return len(self._pyobj)

    def __iter__(self):
        """
        Returns an iterator over the individuals in the population.
        :return: An iterator over the individuals in the population.
        """
        for phenotype in self._pyobj.phenotypes:
            yield Phenotype.from_python(phenotype)

    def __getitem__(self, index: int) -> Phenotype[T]:
        """
        Returns the Phenotype at the specified index.
        :param index: The index of the Phenotype to retrieve.
        :return: The Phenotype at the specified index.
        """
        return Phenotype.from_python(self._pyobj[index])

    def gene_type(self) -> GeneType:
        """
        Returns the type of the genes in the population.
        :return: The gene type as a string.
        """
        return GeneType.from_str(self._pyobj.gene_type())
