from typing import Any

from radiate.wrapper import PyObject
from radiate.radiate import PyFront, PyFrontValue


class FrontValue(PyObject[PyFrontValue]):
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


class Front(PyObject[PyFront]):
    """
    Front class that wraps around the PyFront class.
    This class provides a simple interface to access the value of the front.
    """

    def __iter__(self):
        """
        Get an iterator over the members of the front.
        :return: An iterator over the members of the front.
        """
        for member in self.values():
            yield member

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
