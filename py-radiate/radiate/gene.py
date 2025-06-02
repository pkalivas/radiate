from __future__ import annotations

from typing import Tuple
from radiate.radiate import PyGene


class Gene:

    def __init__(self, py_gene: PyGene):
        """
        Initialize the Gene class.
        This class is a placeholder for gene-related functionality.
        """
        self.py_gene = py_gene

    def __eq__(self, other: object) -> bool:
        """
        Check equality with another Gene instance.
        :param other: Another Gene instance to compare with.
        :return: True if both instances are equal, False otherwise.
        """
        if not isinstance(other, Gene):
            return False
        return self.py_gene == other.py_gene
    
    def __repr__(self) -> str:
        """
        Return a string representation of the Gene instance.
        :return: String representation of the Gene instance.
        """
        return f"Gene({self.py_gene.__repr__()})"

    def __hash__(self) -> int:
        """
        Return a hash of the Gene instance.
        :return: Hash of the Gene instance.
        """
        return hash(self.py_gene)
    


    @staticmethod
    def float(allele: float | None = None,
              value_range: Tuple[float, float] | None = None,
              bound_range: Tuple[float, float] | None = None) -> "Gene":
        """
        Create a float gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as a float gene.
        """
        return Gene(PyGene.float(allele=allele, range=value_range, bounds=bound_range))
    
    @staticmethod
    def int(allele: int | None = None,
            value_range: Tuple[int, int] | None = None,
            bound_range: Tuple[int, int] | None = None) -> "Gene":
        """
        Create an integer gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as an integer gene.
        """
        return Gene(PyGene.int(allele=allele, range=value_range, bounds=bound_range))
    
    @staticmethod
    def bit(allele: bool | None = None) -> "Gene":
        """
        Create a bit gene with an optional allele.
        :param allele: Initial value of the gene.
        :return: A new Gene instance configured as a bit gene.
        """
        return Gene(PyGene.bit(allele=allele))

    @staticmethod
    def char(allele: str | None = None, char_set: set[str] | None = None) -> "Gene":
        """
        Create a character gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as a character gene.
        """
        return Gene(PyGene.char(allele=allele, char_set=list(char_set) if char_set else None))