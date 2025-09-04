from __future__ import annotations

from radiate.genome.phenotype import Phenotype
from radiate.genome.population import Population
from radiate.wrapper import PyObject
from radiate.radiate import PySpecies


class Species[T](PyObject[PySpecies]):
    """
    Represents a species in a population.
    """

    def __init__(self, mascot: Phenotype[T], generation: int = 0):
        super().__init__()
        self._pyobj = PySpecies(mascot.__backend__(), generation)

    def __repr__(self):
        return self._pyobj.__repr__()

    def add_individual(self, individual: Phenotype[T]):
        """
        Adds an individual to the species.

        :param individual: The individual to add.
        """
        self._pyobj.add_individual(individual.__backend__())

    def population(self) -> Population[T]:
        """
        Returns the population of the species.

        :return: Population of the species.
        """
        return Population.from_rust(self._pyobj.population())
