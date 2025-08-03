from __future__ import annotations

from typing import Optional, Tuple, Set
from enum import Enum

from radiate.radiate import PyGene

class GeneType(Enum):
    FLOAT = "FloatGene"
    INT = "IntGene"
    BIT = "BitGene"
    CHAR = "CharGene"
    GRAPH = "GraphNode"
    TREE = "TreeNode"
    PERMUTATION = "PermutationGene"

    @staticmethod
    def all() -> Set[GeneType]:
        return {
            GeneType.FLOAT,
            GeneType.INT,
            GeneType.BIT,
            GeneType.CHAR,
            GeneType.GRAPH,
            GeneType.TREE,
            GeneType.PERMUTATION,
        }
    
    @staticmethod
    def core() -> Set[GeneType]:
        return {
            GeneType.FLOAT,
            GeneType.INT,
            GeneType.BIT,
            GeneType.CHAR,
            GeneType.PERMUTATION,
        }
    
    @staticmethod
    def from_str(gene_type: str) -> GeneType:
        type_lower = str(gene_type).lower()
        for gene in GeneType:
            if gene.value.lower() == type_lower:
                return gene
        raise ValueError(f"Invalid gene type: {gene_type}")


class Gene[T]:
    def __init__(self, py_gene: PyGene) -> None:
        self.__inner = py_gene

    def __eq__(self, other: object) -> bool:
        """
        Check equality with another Gene instance.
        :param other: Another Gene instance to compare with.
        :return: True if both instances are equal, False otherwise.
        """
        if not isinstance(other, Gene):
            return False
        return self.__inner == other.__inner

    def __repr__(self) -> str:
        """
        Return a string representation of the Gene instance.
        :return: String representation of the Gene instance.
        """
        return f"Gene({self.__inner.__repr__()})"

    def __hash__(self) -> int:
        """
        Return a hash of the Gene instance.
        :return: Hash of the Gene instance.
        """
        return hash(self.__inner)

    def py_gene(self) -> PyGene:
        """
        Get the underlying PyGene instance.
        :return: The PyGene instance associated with this Gene.
        """
        return self.__inner

    def gene_type(self) -> GeneType:
        """
        Get the type of the gene.
        :return: The type of the gene as a string.
        """
        return GeneType.from_str(self.__inner.gene_type().name())

    def allele(self) -> T:
        """
        Get the allele of the gene.
        :return: The allele of the gene, which can be a float, int, bool, str, or None.
        """
        return self.__inner.allele()

    @staticmethod
    def float(
        allele: Optional[float] = None,
        *,
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
    ) -> Gene[float]:
        """
        Create a float gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as a float gene.

        Example
        --------
        >>> rd.Gene.float(allele=5.0, value_range=(-10.0, 10.0), bound_range=(-20.0, 20.0))
        Gene(5.0)
        """
        return Gene(PyGene.float(allele=allele, range=value_range, bounds=bound_range))

    @staticmethod
    def int(
        allele: int | None = None,
        *,
        value_range: Tuple[int, int] | None = None,
        bound_range: Tuple[int, int] | None = None,
    ) -> Gene[int]:
        """
        Create an integer gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as an integer gene.

        Example
        --------
        >>> rd.Gene.int(allele=5, value_range=(0, 10), bound_range=(-5, 15))
        Gene(5)
        """
        return Gene(PyGene.int(allele=allele, range=value_range, bounds=bound_range))

    @staticmethod
    def bit(allele: bool | None = None) -> Gene[bool]:
        """
        Create a bit gene with an optional allele.
        :param allele: Initial value of the gene.
        :return: A new Gene instance configured as a bit gene.

        Example
        --------
        >>> rd.Gene.bit(allele=True)
        Gene(True)
        """
        return Gene(PyGene.bit(allele=allele))

    @staticmethod
    def char(allele: str | None = None, char_set: set[str] | None = None) -> Gene[str]:
        """
        Create a character gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as a character gene.

        Example
        --------
        >>> rd.Gene.char(allele='a', char_set={'a', 'b', 'c'})
        Gene(a)
        """
        return Gene(
            PyGene.char(allele=allele, char_set=list(char_set) if char_set else None)
        )
