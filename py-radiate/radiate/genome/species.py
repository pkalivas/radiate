from __future__ import annotations

from .._bridge import RsObject
from .phenotype import Phenotype


class Species[T](RsObject):
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

    def mascot(self) -> Phenotype[T]:
        """
        Returns the mascot of the species.

        :return: Mascot of the species.
        """
        return self.try_get_cache(
            "mascot_cache",
            lambda: Phenotype.from_rust(self.__backend__().mascot),
        )

    def generation(self) -> int:
        """
        Returns the generation of the species.

        :return: Generation of the species.
        """
        return self.__backend__().generation

    def stagnation(self) -> int:
        """
        Returns the stagnation of the species.

        :return: Stagnation of the species.
        """
        return self.__backend__().stagnation

    def score(self) -> list[float] | None:
        """
        Returns the score of the species.

        :return: Score of the species.
        """
        return self.__backend__().score()
