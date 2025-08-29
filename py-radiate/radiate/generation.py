from radiate.genome.ecosystem import Ecosystem
from radiate.genome.species import Species
from .genome import Population
from radiate.radiate import PyGeneration


class Generation:
    """
    Generation class that wraps around the PyGeneration class.
    This class provides a simple interface to access the value of the generation.
    """

    def __init__(self, py_generation: PyGeneration):
        self.inner = py_generation

    def __repr__(self):
        return f"{self.inner.__repr__()}"

    def score(self):
        """
        Get the fitness of the generation.
        :return: The fitness of the generation.
        """
        return self.inner.score()

    def index(self):
        """
        Get the index of the generation.
        :return: The index of the generation.
        """
        return self.inner.index()

    def value(self):
        """
        Get the value of the generation.
        :return: The value of the generation.
        """
        return self.inner.value()

    def metrics(self):
        """
        Get the metrics of the generation.
        :return: The metrics of the generation.
        """
        return self.inner.metrics()

    def population(self) -> Population:
        """
        Get the population of the generation.
        :return: The population of the generation.
        """
        return Population.from_rust(self.inner.population())

    def species(self) -> list[Species] | None:
        """
        Get the species of the generation.
        :return: The species of the generation.
        """
        species = self.inner.species()
        if species is None:
            return None
        return [Species.from_rust(s) for s in species]

    def ecosystem(self) -> Ecosystem:
        """
        Get the ecosystem of the generation.
        :return: The ecosystem of the generation.
        """
        return Ecosystem.from_rust(self.inner.ecosystem())

    def duration(self):
        """
        Get the duration of the generation.
        :return: The duration of the generation.
        """
        return self.inner.duration()
