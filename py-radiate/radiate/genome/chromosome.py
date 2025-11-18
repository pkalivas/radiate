from __future__ import annotations

from typing import Iterable, TYPE_CHECKING
from radiate.wrapper import PyObject
from radiate.radiate import PyChromosome
from .gene import Gene
from radiate.genome import gene

if TYPE_CHECKING:
    from radiate.genome import GeneType


class Chromosome[T](PyObject[PyChromosome]):
    """
    Represents a chromosome in a genome.
    """

    def __init__(
        self,
        genes: Iterable[Gene[T]] | Gene[T] | None = None,
    ):
        super().__init__()

        if genes is None:
            return

        if isinstance(genes, Gene):
            self._pyobj = PyChromosome([genes.__backend__()])
        if isinstance(genes, Iterable):
            self._pyobj = PyChromosome(list(map(lambda g: g.__backend__(), genes)))
        else:
            raise TypeError("genes must be a Gene instance or a list of Gene instances")

    def __repr__(self):
        return self.__backend__().__repr__()

    def __len__(self):
        """
        Returns the length of the chromosome.
        :return: Length of the chromosome.
        """
        return self.__backend__().__len__()

    def __getitem__(self, index: int) -> Gene[T]:
        """
        Returns the gene at the specified index.
        :param index: Index of the gene to retrieve.
        :return: Gene instance at the specified index.
        """
        return Gene.from_rust(self._pyobj[index])

    def __iter__(self):
        """
        Creates an iterator over a View of the chromosome. This means that
        each gene returned by the iterator is a view into the original chromosome and
        is not a copy, so changes to the gene will affect the original chromosome.

        :return: An iterator over the genes in the chromosome.
        """
        for i in self.__backend__():
            yield Gene.from_rust(i)

    def gene_type(self) -> "GeneType":
        from . import GeneType

        return GeneType.from_str(self.__backend__().gene_type())


def int(
    length: int,
    init_range: tuple[int, int] | None = None,
    bounds: tuple[int, int] | None = None,
) -> Chromosome[int]:
    """
    Create an integer chromosome with specified length and optional parameters.
    :param length: Length of the chromosome.
    :param allele: Initial value of the gene.
    :param init_range: Minimum and maximum value for the gene.
    :param bounds: Minimum and maximum bound for the gene.
    :return: A new Chromosome instance configured as an integer chromosome.

    Example
    --------
    >>> rd.Chromosome.int(length=3, init_range=(0, 10), bounds=(-5, 15))
    Chromosome(genes=[0, 5, 10])
    """
    genes = [gene.int(init_range=init_range, bounds=bounds) for _ in range(length)]
    return Chromosome(genes=genes)


def bit(length: int) -> Chromosome[bool]:
    """
    Create a bit chromosome with specified length and optional allele.
    :param length: Length of the chromosome.
    :param allele: Initial value of the gene.
    :return: A new Chromosome instance configured as a bit chromosome.

    Example
    --------
    >>> rd.chromosome.bit(length=4)
    Chromosome(genes=[True, False, True, False])
    """
    genes = [gene.bit() for _ in range(length)]
    return Chromosome(genes=genes)


def char(length: int, char_set: set[str] | None = None) -> Chromosome[str]:
    """
    Create a character chromosome with specified length and optional character set.
    :param length: Length of the chromosome.
    :param char_set: Set of characters to choose from.
    :return: A new Chromosome instance configured as a character chromosome.

    Example
    --------
    >>> rd.chromosome.char(length=5, char_set={'a', 'b', 'c'})
    Chromosome(genes=[a, b, c, a, b])
    """
    genes = [gene.char(char_set=char_set) for _ in range(length)]
    return Chromosome(genes=genes)


def float(
    length: int,
    init_range: tuple[float, float] | None = None,
    bounds: tuple[float, float] | None = None,
) -> Chromosome[float]:
    """
    Create a float chromosome with specified length and optional parameters.
    :param length: Length of the chromosome.
    :param allele: Initial value of the gene.
    :param init_range: Minimum and maximum value for the gene.
    :param bounds: Minimum and maximum bound for the gene.
    :return: A new Chromosome instance configured as a float chromosome.

    Example
    --------
    >>> rd.chromosome.float(length=5, init_range=(0.0, 10.0), bounds=(-5.0, 15.0))
    Chromosome(genes=[0.0, 2.5, 5.0, 7.5, 10.0])
    """
    genes = [gene.float(init_range=init_range, bounds=bounds) for _ in range(length)]
    return Chromosome(genes=genes)
