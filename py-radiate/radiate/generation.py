from datetime import timedelta

from radiate.genome.ecosystem import Ecosystem
from radiate.genome.species import Species
from radiate.wrapper import PyObject
from radiate.metrics import MetricSet
from .genome import Population
from radiate.radiate import PyGeneration


class Generation[T](PyObject[PyGeneration]):
    """
    Generation class that wraps around the PyGeneration class.
    This class provides a simple interface to access the value of the generation.
    """

    def __repr__(self):
        return f"{self.__backend__().__repr__()}"

    def score(self) -> list[float]:
        """
        Get the fitness of the generation.
        :return: The fitness of the generation.
        """
        return self.__backend__().score()

    def index(self) -> int:
        """
        Get the index of the generation.
        :return: The index of the generation.
        """
        return self.__backend__().index()

    def value(self) -> T:
        """
        Get the value of the generation.
        :return: The value of the generation.
        """
        return self.__backend__().value()

    def metrics(self) -> MetricSet:
        """
        Get the metrics of the generation.
        :return: The metrics of the generation.
        """
        return MetricSet.from_rust(self.__backend__().metrics())

    def objective(self) -> list[str] | str:
        """
        Get the objective names of the generation.
        :return: The objective names of the generation.
        """
        obj = self.__backend__().objective()
        if len(obj) == 1:
            return obj[0]
        return obj

    def population(self) -> Population:
        """
        Get the population of the generation.
        :return: The population of the generation.
        """
        return Population.from_rust(self.__backend__().population())

    def species(self) -> list[Species] | None:
        """
        Get the species of the generation.
        :return: The species of the generation.
        """
        species = self.__backend__().species()
        if species is None:
            return None
        return [Species.from_rust(s) for s in species]

    def ecosystem(self) -> Ecosystem:
        """
        Get the ecosystem of the generation.
        :return: The ecosystem of the generation.
        """
        return Ecosystem.from_rust(self.__backend__().ecosystem())

    def duration(self) -> timedelta:
        """
        Get the duration of the generation.
        :return: The duration of the generation.
        """
        return self.__backend__().duration()
