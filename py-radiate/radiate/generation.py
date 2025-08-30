from typing import Any

from datetime import timedelta

from radiate.genome.ecosystem import Ecosystem
from radiate.genome.species import Species
from radiate.wrapper import PyObject
from .genome import Population
from radiate.radiate import PyGeneration


class Generation[T](PyObject[PyGeneration]):
    """
    Generation class that wraps around the PyGeneration class.
    This class provides a simple interface to access the value of the generation.
    """

    def __repr__(self):
        return f"{self._pyobj.__repr__()}"

    def score(self) -> list[float]:
        """
        Get the fitness of the generation.
        :return: The fitness of the generation.
        """
        return self._pyobj.score()

    def index(self) -> int:
        """
        Get the index of the generation.
        :return: The index of the generation.
        """
        return self._pyobj.index()

    def value(self) -> T:
        """
        Get the value of the generation.
        :return: The value of the generation.
        """
        return self._pyobj.value()

    def metrics(self) -> dict[str, Any]:
        """
        Get the metrics of the generation.
        :return: The metrics of the generation.
        """
        return self._pyobj.metrics()

    def population(self) -> Population:
        """
        Get the population of the generation.
        :return: The population of the generation.
        """
        return Population.from_rust(self._pyobj.population())

    def species(self) -> list[Species] | None:
        """
        Get the species of the generation.
        :return: The species of the generation.
        """
        species = self._pyobj.species()
        if species is None:
            return None
        return [Species.from_rust(s) for s in species]

    def ecosystem(self) -> Ecosystem:
        """
        Get the ecosystem of the generation.
        :return: The ecosystem of the generation.
        """
        return Ecosystem.from_rust(self._pyobj.ecosystem())

    def duration(self) -> timedelta:
        """
        Get the duration of the generation.
        :return: The duration of the generation.
        """
        return self._pyobj.duration()
