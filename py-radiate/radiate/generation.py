from datetime import timedelta

from radiate.genome.ecosystem import Ecosystem
from radiate.genome.species import Species
from radiate.wrapper import PyObject
from radiate.metrics import MetricSet
from radiate.front import Front
from .genome import Population
from radiate.radiate import PyGeneration


class Generation[T](PyObject[PyGeneration]):
    """
    Generation class that wraps around the PyGeneration class.
    This class provides a simple interface to access the value of the generation.
    """

    def __repr__(self):
        return f"{self.__backend__().__repr__()}"

    def to_json(self) -> str:
        """
        Serialize the generation to a JSON string.
        :return: The JSON string representation of the generation.
        """
        return self.__backend__().to_json()

    @staticmethod
    def from_json(json_str: str) -> "Generation":
        """
        Deserialize a JSON string to a Generation object.
        :param json_str: The JSON string representation of the generation.
        :return: A Generation object.
        """
        return Generation.from_rust(PyGeneration.from_json(json_str))

    def score(self) -> list[float]:
        """
        Get the fitness of the generation.
        :return: The fitness of the generation.
        """
        return self.try_get_cache("score_cache", lambda: self.__backend__().score())

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

        def _get_value():
            from radiate.radiate import PyGraph, PyTree
            from .gp.tree import Tree
            from .gp.graph import Graph

            val = self.__backend__().value()

            if isinstance(val, PyGraph):
                return Graph.from_rust(val)
            elif isinstance(val, PyTree):
                return Tree.from_rust(val)
            else:
                return val

        return self.try_get_cache("value_cache", _get_value)

    def front(self) -> Front:
        """
        Get the Pareto front of the generation.
        :return: The Pareto front of the generation.
        """
        return self.try_get_cache(
            "front_cache", lambda: Front.from_rust(self.__backend__().front())
        )

    def metrics(self) -> MetricSet:
        """
        Get the metrics of the generation.
        :return: The metrics of the generation.
        """
        return self.try_get_cache(
            "metrics_cache", lambda: MetricSet.from_rust(self.__backend__().metrics())
        )

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
        return self.try_get_cache(
            "population_cache",
            lambda: Population.from_rust(self.__backend__().population()),
        )

    def species(self) -> list[Species] | None:
        """
        Get the species of the generation.
        :return: The species of the generation.
        """

        def _get_species():
            species = self.__backend__().species()
            if species is None:
                return None
            return [Species.from_rust(s) for s in species]

        return self.try_get_cache("species_cache", _get_species)

    def ecosystem(self) -> Ecosystem:
        """
        Get the ecosystem of the generation.
        :return: The ecosystem of the generation.
        """
        return self.try_get_cache(
            "ecosystem_cache",
            lambda: Ecosystem.from_rust(self.__backend__().ecosystem()),
        )

    def duration(self) -> timedelta:
        """
        Get the duration of the generation.
        :return: The duration of the generation.
        """
        return self.try_get_cache(
            "duration_cache", lambda: self.__backend__().duration()
        )
