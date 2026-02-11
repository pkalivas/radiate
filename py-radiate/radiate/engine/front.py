from typing import Any

from radiate._bridge.wrapper import RsObject
from radiate.radiate import PyFront, PyFrontValue
from radiate.genome.genotype import Genotype
from radiate.genome.phenotype import Phenotype


class FrontValue(RsObject[PyFrontValue]):
    """
    FrontValue class that wraps around the PyFrontValue class.
    This class provides a simple interface to access the value of the front value.
    """

    def genotype(self) -> Any:
        """
        Get the genotype of the front value.
        :return: The genotype of the front value.
        """
        return self.try_get_cache("genotype_cache", self.__backend__().genotype)

    def score(self) -> list[float]:
        """
        Get the score of the front value.
        :return: The score of the front value.
        """
        return self.try_get_cache("score_cache", self.__backend__().score)


class Front(RsObject[PyFront]):
    """
    Front class that wraps around the PyFront class.
    This class provides a simple interface to access the value of the front.
    """

    def __init__(self, objectives: list[str], range: tuple[int, int]):
        super().__init__()
        self._pyobj = PyFront(range, objectives)

    def __iter__(self):
        """
        Get an iterator over the members of the front.
        :return: An iterator over the members of the front.
        """
        for member in self.values():
            yield member

    def __len__(self):
        """
        Get the length of the front.
        :return: The length of the front.
        """
        return len(self.__backend__())

    def __getitem__(self, index: int | slice) -> FrontValue | list[FrontValue]:
        """
        Get a member of the front by index.
        :param index: The index of the member.
        :return: The member at the given index.
        """
        return self.values()[index]

    def values(self) -> list[FrontValue]:
        """
        Get the values of the front.
        :return: The values of the front.
        """
        return self.try_get_cache(
            "values_cache",
            lambda: [FrontValue.from_rust(v) for v in self.__backend__().values()],
        )

    def remove_outliers(self, trim: float = 0.01) -> int | None:
        """
        Remove outliers from the front.
        :param trim: The percentage of outliers to remove (between 0 and 0.5).
        """
        if trim <= 0.0 or trim >= 0.5:
            raise ValueError("Trim must be between 0 and 0.5.")
        result = self.__backend__().remove_outliers(trim)
        self.try_invalidate_cache("values_cache")
        return result

    def entropy(self) -> float:
        """
        Get the entropy of the front.
        :return: The entropy of the front.
        """
        return self.try_get_cache("entropy_cache", self.__backend__().entropy)

    def crowding_distance(self) -> list[float] | None:
        """
        Get the crowding distance of the front.
        :return: The crowding distance of the front, or None if it cannot be calculated.
        """
        return self.try_get_cache(
            "crowding_distance_cache",
            lambda: self.__backend__().crowding_distance(),
        )

    def add(
        self, items: list[Phenotype] | list[tuple[Genotype, list[float]]]
    ) -> dict[str, Any] | None:
        """
        Add items to the front.
        :param items: A list of Phenotypes or a list of tuples containing Genotypes and their scores.
        """
        if not items:
            return None

        to_add = []
        if isinstance(items, list):
            if all(isinstance(item, Phenotype) for item in items):
                to_add = [item.__backend__() for item in items]
            elif all(
                isinstance(item, tuple)
                and len(item) == 2
                and isinstance(item[0], Genotype)
                and isinstance(item[1], (list, float))
                for item in items
            ):
                for item in items:
                    genotype, score = item
                    if isinstance(score, float):
                        score = [score]
                    to_add.append(
                        Phenotype(genotype=genotype, score=score).__backend__()
                    )
            else:
                raise ValueError(
                    "Items must be a list of Phenotypes or a list of tuples containing Genotypes and their scores."
                )
        else:
            raise ValueError(
                "Items must be a list of Phenotypes or a list of tuples containing Genotypes and their scores."
            )

        self.try_invalidate_cache("values_cache")
        self.try_invalidate_cache("fronts_cache")

        return self.__backend__().add(to_add)

    def fronts(self) -> list["Front"]:
        """
        Get the fronts of the front.
        :return: The fronts of the front.
        """

        def _get_fronts():
            fronts = self.__backend__().fronts()
            return [Front.from_rust(f) for f in fronts]

        return self.try_get_cache("fronts_cache", _get_fronts)
