from __future__ import annotations

from typing import Iterable, Iterator
from radiate.wrapper import PyObject
from radiate.radiate import PyChromosome, PyGene
from .gene import Gene, GeneType
from radiate.genome import gene


class Chromosome[T](PyObject[PyChromosome]):
    """
    Represents a chromosome in a genome.
    """

    def __init__(
        self,
        genes: Iterable[Gene[T]] | Gene[T] | None = None,
    ):
        super().__init__()

        # Default to an empty chromosome when no genes are provided
        self._pyobj = PyChromosome([])

        if genes is None:
            return

        if isinstance(genes, Gene):
            self._pyobj = PyChromosome([genes.to_python()])
        elif isinstance(genes, Iterable):
            self._pyobj = PyChromosome(list(map(lambda g: g.to_python(), genes)))
        else:
            raise TypeError("genes must be a Gene instance or an iterable of Gene instances")

    def __repr__(self):
        return repr(self._pyobj)

    def __len__(self):
        """
        Returns the length of the chromosome.
        :return: Length of the chromosome.
        """
        return len(self._pyobj)

    def __getitem__(self, index: int | slice) -> Gene[T] | Chromosome[T]:
        """
        Returns the gene at the specified index.
        :param index: Index of the gene to retrieve.
        :return: Gene instance at the specified index.
        """
        item = self._pyobj[index]
        if isinstance(index, slice):
            return Chromosome.from_python(item)
        return Gene.from_python(item)

    def __setitem__(self, index: int | slice, gene: Gene[T] | Chromosome[T] | T):
        """
        Sets the gene at the specified index.
        :param index: Index of the gene to set.
        :param gene: Gene instance to set at the specified index.
        """
        if isinstance(gene, Chromosome) or isinstance(gene, Gene):
            self._pyobj[index] = gene.to_python()
        elif isinstance(gene, PyChromosome) or isinstance(gene, PyGene):
            self._pyobj[index] = gene
        else:
            # Setting a raw allele; handle negative indices explicitly and
            # disallow slice assignment with raw values.
            if isinstance(index, slice):
                raise TypeError("slice assignment requires a Chromosome with matching length")
            idx = index
            if idx < 0:
                idx += len(self)
            self._pyobj.set_allele(idx, gene)

    def __iter__(self) -> Iterator[Gene[T]]:
        """
        Returns an iterator over the genes in the chromosome.
        :return: An iterator over the genes in the chromosome.
        """
        for i in range(len(self)):
            yield Gene.from_python(self._pyobj[i])

    def gene_type(self) -> GeneType:
        return GeneType.from_str(self._pyobj.gene_type().name())

    def alleles(self) -> list[T]:
        """
        Collect the allele values for all genes in the chromosome.
        :return: A list of allele values of type T.
        """
        return [g.allele() for g in self]


def int(
    length: int,
    *,
    init_range: tuple[int, int] | None = None,
    bounds: tuple[int, int] | None = None,
) -> Chromosome[int]:
    """
    Create an integer chromosome with specified length and optional parameters.
    :param length: Length of the chromosome.
    :param init_range: Minimum and maximum initial value for each gene.
    :param bounds: Minimum and maximum bound for each gene.
    :return: A new Chromosome instance configured as an integer chromosome.

    Example
    --------
    >>> rd.chromosome.int(length=3, init_range=(0, 10), bounds=(-5, 15))
    Chromosome(genes=[0, 5, 10])
    """
    genes = [gene.int(init_range=init_range, bounds=bounds) for _ in range(length)]
    return Chromosome(genes=genes)


def bit(length: int) -> Chromosome[bool]:
    """
    Create a bit chromosome with specified length and optional allele.
    :param length: Length of the chromosome.
    :return: A new Chromosome instance configured as a bit chromosome.

    Example
    --------
    >>> rd.chromosome.bit(length=4)
    Chromosome(genes=[True, False, True, False])
    """
    genes = [gene.bit() for _ in range(length)]
    return Chromosome(genes=genes)


def char(length: int, char_set: Iterable[str] | str | None = None) -> Chromosome[str]:
    """
    Create a character chromosome with specified length and optional character set.
    :param length: Length of the chromosome.
    :param char_set: Iterable or string of characters to choose from.
    :return: A new Chromosome instance configured as a character chromosome.

    Example
    --------
    >>> rd.chromosome.char(length=5, char_set={'a', 'b', 'c'})
    Chromosome(genes=[a, b, c, a, b])
    """
    genes = [gene.char(char_set=set(char_set) if char_set is not None else None) for _ in range(length)]
    return Chromosome(genes=genes)


def float(
    length: int,
    *,
    init_range: tuple[float, float] | None = None,
    bounds: tuple[float, float] | None = None,
) -> Chromosome[float]:
    """
    Create a float chromosome with specified length and optional parameters.
    :param length: Length of the chromosome.
    :param init_range: Minimum and maximum initial value for each gene.
    :param bounds: Minimum and maximum bound for each gene.
    :return: A new Chromosome instance configured as a float chromosome.

    Example
    --------
    >>> rd.chromosome.float(length=5, init_range=(0.0, 10.0), bounds=(-5.0, 15.0))
    Chromosome(genes=[0.0, 2.5, 5.0, 7.5, 10.0])
    """
    genes = [gene.float(init_range=init_range, bounds=bounds) for _ in range(length)]
    return Chromosome(genes=genes)
