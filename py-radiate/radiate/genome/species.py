from __future__ import annotations

from radiate._bridge.wrapper import RsObject


class Species(RsObject):
    """
    Represents a species in a population.
    """

    def __repr__(self):
        return self.__backend__().__repr__()

    def population(self) -> set[int]:
        """
        Returns the population of the species.

        :return: Population of the species.
        """
        return self.try_get_cache(
            "population_cache",
            lambda: set(self.__backend__().population),
        )
