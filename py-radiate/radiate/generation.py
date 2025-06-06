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
        return Population(self.inner.population())
