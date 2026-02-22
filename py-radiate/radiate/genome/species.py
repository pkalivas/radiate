from __future__ import annotations

from radiate._bridge.wrapper import RsObject

from .population import Population


class Species[T](RsObject):
    """
    Represents a species in a population.
    """

    def __repr__(self):
        return self.__backend__().__repr__()

    def population(self) -> Population[T]:
        """
        Returns the population of the species.

        :return: Population of the species.
        """
        return self.try_get_cache(
            "population_cache",
            lambda: Population.from_rust(self.__backend__().population),
        )
