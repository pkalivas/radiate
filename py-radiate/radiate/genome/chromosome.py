from __future__ import annotations

from typing import Tuple, List
from radiate.genome.wrapper import PythonWrapper
from radiate.radiate import PyChromosome
from .gene import BitGene, CharGene, FloatGene, Gene, IntGene


class Chromosome[T](PythonWrapper[PyChromosome]):
    """
    Represents a chromosome in a genome.
    """

    def __init__(
        self,
        genes: List[Gene[T]] | Gene[T] | None = None,
    ):
        """
        Initializes a Chromosome instance.

        :param gene_type: The type of the genes in the chromosome.
        :param length: The length of the chromosome.
        """
        super().__init__()
        if genes is None:
            return

        if isinstance(genes, Gene):
            self._pyobj = PyChromosome([genes.to_python()])
        if isinstance(genes, list):
            self._pyobj = PyChromosome([gene.to_python() for gene in genes])
        else:
            raise TypeError("genes must be a Gene instance or a list of Gene instances")

    def __repr__(self):
        return self._pyobj.__repr__()

    def __len__(self):
        """
        Returns the length of the chromosome.
        :return: Length of the chromosome.
        """
        return self._pyobj.__len__()

    def __getitem__(self, index: int) -> Gene[T]:
        """
        Returns the gene at the specified index.
        :param index: Index of the gene to retrieve.
        :return: Gene instance at the specified index.
        """
        return Gene.from_python(self._pyobj[index])

    def __iter__(self):
        """
        Returns an iterator over the genes in the chromosome.
        :return: An iterator over the genes in the chromosome.
        """
        for gene in self._pyobj.genes:
            yield Gene.from_python(gene)

    def gene_type(self) -> str:
        return self._pyobj.gene_type()
    
    @staticmethod
    def float(
        length: int,
        *,
        value_range: Tuple[float, float] | None = None,
        bound_range: Tuple[float, float] | None = None,
    ) -> Chromosome[float]:
        """
        Create a float chromosome with specified length and optional parameters.
        :param length: Length of the chromosome.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Chromosome instance configured as a float chromosome.

        Example
        --------
        >>> rd.Chromosome.float(length=5, value_range=(0.0, 10.0), bound_range=(-5.0, 15.0))
        Chromosome(genes=[0.0, 2.5, 5.0, 7.5, 10.0])
        """
        genes = [
            FloatGene(value_range=value_range, bound_range=bound_range)
            for _ in range(length)
        ]
        return Chromosome(genes=genes)

    @staticmethod
    def int(
        length: int,
        *,
        value_range: Tuple[int, int] | None = None,
        bound_range: Tuple[int, int] | None = None,
    ) -> Chromosome[int]:
        """
        Create an integer chromosome with specified length and optional parameters.
        :param length: Length of the chromosome.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Chromosome instance configured as an integer chromosome.

        Example
        --------
        >>> rd.Chromosome.int(length=3, value_range=(0, 10), bound_range=(-5, 15))
        Chromosome(genes=[0, 5, 10])
        """
        genes = [
            IntGene(value_range=value_range, bound_range=bound_range)
            for _ in range(length)
        ]
        return Chromosome(genes=genes)

    @staticmethod
    def bit(length: int) -> Chromosome[bool]:
        """
        Create a bit chromosome with specified length and optional allele.
        :param length: Length of the chromosome.
        :param allele: Initial value of the gene.
        :return: A new Chromosome instance configured as a bit chromosome.

        Example
        --------
        >>> rd.Chromosome.bit(length=4)
        Chromosome(genes=[True, False, True, False])
        """
        genes = [BitGene() for _ in range(length)]
        return Chromosome(genes=genes)

    @staticmethod
    def char(length: int, char_set: set[str] | None = None) -> Chromosome[str]:
        """
        Create a character chromosome with specified length and optional character set.
        :param length: Length of the chromosome.
        :param char_set: Set of characters to choose from.
        :return: A new Chromosome instance configured as a character chromosome.

        Example
        --------
        >>> rd.Chromosome.char(length=5, char_set={'a', 'b', 'c'})
        Chromosome(genes=[a, b, c, a, b])
        """
        genes = [CharGene(char_set=char_set) for _ in range(length)]
        return Chromosome(genes=genes)
