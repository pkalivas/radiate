from __future__ import annotations

from typing import Tuple, List
from radiate.radiate import PyChromosome
from .gene import Gene


class Chromosome:
    """
    Represents a chromosome in a genome.
    """

    def __init__(
        self,
        chromosome: PyChromosome | None = None,
        genes: List[Gene] | Gene | None = None,
    ):
        """
        Initializes a Chromosome instance.

        :param gene_type: The type of the genes in the chromosome.
        :param length: The length of the chromosome.
        """
        if chromosome is not None:
            if isinstance(chromosome, PyChromosome):
                self.__inner = chromosome
            else:
                raise TypeError("chromosome must be an instance of PyChromosome")
        elif genes is not None:
            if isinstance(genes, Gene):
                self.__inner = PyChromosome([genes.py_gene()])
            if isinstance(genes, list):
                if all(isinstance(gene, Gene) for gene in genes):
                    self.__inner = PyChromosome([gene.py_gene() for gene in genes])
                else:
                    raise ValueError("All genes must be instances of Gene")
            else:
                raise TypeError(
                    "genes must be a Gene instance or a list of Gene instances"
                )
        else:
            raise ValueError("Either chromosome or genes must be provided")

    def __repr__(self):
        return f"Chromosome(genes={self.__inner.genes})"

    def __len__(self):
        """
        Returns the length of the chromosome.
        :return: Length of the chromosome.
        """
        return len(self.__inner.genes)
    
    def __eq__(self, value):
        if not isinstance(value, Chromosome):
            return False
        if len(self) != len(value):
            return False
        return all(a == b for a, b in zip(self.__inner.genes, value.__inner.genes))

    def py_chromosome(self) -> PyChromosome:
        """
        Returns the underlying PyChromosome instance.
        :return: The PyChromosome instance associated with this Chromosome.
        """
        return self.__inner

    def genes(self) -> List[Gene]:
        """
        Returns the genes of the chromosome.
        :return: A list of Gene instances.
        """
        return [Gene(gene) for gene in self.__inner.genes]

    @staticmethod
    def float(
        length: int,
        value_range: Tuple[float, float] | None = None,
        bound_range: Tuple[float, float] | None = None,
    ) -> "Chromosome":
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
            Gene.float(value_range=value_range, bound_range=bound_range)
            for _ in range(length)
        ]
        return Chromosome(genes=genes)

    @staticmethod
    def int(
        length: int,
        value_range: Tuple[int, int] | None = None,
        bound_range: Tuple[int, int] | None = None,
    ) -> "Chromosome":
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
            Gene.int(value_range=value_range, bound_range=bound_range)
            for _ in range(length)
        ]
        return Chromosome(genes=genes)

    @staticmethod
    def bit(length: int) -> "Chromosome":
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
        genes = [Gene.bit() for _ in range(length)]
        return Chromosome(genes=genes)

    @staticmethod
    def char(length: int, char_set: set[str] | None = None) -> "Chromosome":
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
        genes = [Gene.char(char_set=char_set) for _ in range(length)]
        return Chromosome(genes=genes)
