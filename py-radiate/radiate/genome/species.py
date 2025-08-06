from __future__ import annotations

from radiate.genome.phenotype import Phenotype
from radiate.genome.population import Population
from radiate.genome.wrapper import PythonWrapper
from radiate.radiate import PySpecies


class Species[T](PythonWrapper[PySpecies]):
    """
    Represents a species in a population.
    """

    def __init__(self, mascot: Phenotype[T], generation: int = 0):
        super().__init__()
        self._pyobj = PySpecies(mascot.to_python(), generation)

    def __repr__(self):
        return self._pyobj.__repr__()

    def add_individual(self, individual: Phenotype[T]):
        """
        Adds an individual to the species.

        :param individual: The individual to add.
        """
        self._pyobj.add_individual(individual.to_python())

    def population(self) -> Population[T]:
        """
        Returns the population of the species.

        :return: Population of the species.
        """
        return Population.from_python(self._pyobj.population())
