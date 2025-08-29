from __future__ import annotations

from typing import Callable
from enum import Enum

from radiate.radiate import PyGene
from radiate.wrapper import PyObject


class GeneType(Enum):
    FLOAT = "FloatGene"
    INT = "IntGene"
    BIT = "BitGene"
    CHAR = "CharGene"
    PERMUTATION = "PermutationGene"
    GRAPH = "GraphNode"
    TREE = "TreeNode"
    ANY = "AnyGene"

    @staticmethod
    def all() -> set[GeneType]:
        return {
            GeneType.FLOAT,
            GeneType.INT,
            GeneType.BIT,
            GeneType.CHAR,
            GeneType.PERMUTATION,
            GeneType.GRAPH,
            GeneType.TREE,
            GeneType.ANY,
        }

    @staticmethod
    def core() -> set[GeneType]:
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
        match type_lower:
            case "floatgene":
                return GeneType.FLOAT
            case "intgene":
                return GeneType.INT
            case "bitgene":
                return GeneType.BIT
            case "chargene":
                return GeneType.CHAR
            case "permutationgene":
                return GeneType.PERMUTATION
            case "graphnode":
                return GeneType.GRAPH
            case "treenode":
                return GeneType.TREE
            case "anygene":
                return GeneType.ANY
            case _:
                raise ValueError(f"Invalid gene type: {gene_type}")


class Gene[T](PyObject[PyGene]):
    def __init__(self) -> None:
        super().__init__()

    def gene_type(self) -> GeneType:
        """
        Get the type of the gene.
        :return: The type of the gene as a string.
        """
        return GeneType.from_str(self._pyobj.gene_type().name())

    def allele(self) -> T:
        """
        Get the allele of the gene.
        :return: The allele of the gene, which can be a float, int, bool, str, or None.
        """
        return self._pyobj.allele()

    def new_instance(self, allele: T | None = None) -> Gene[T]:
        """
        Set the allele of the gene.
        :param allele: The new allele value, which can be a float, int, bool, str, or None.
        """
        return Gene.from_python(self._pyobj.new_instance(allele))

    def apply(self, f: Callable[[T], T]) -> None:
        """
        Apply a function to the allele of the gene.
        :param f: The function to apply to the allele.
        """
        self._pyobj.apply(f)

    def map(self, f: Callable[[T], T]) -> Gene[T]:
        """
        Map a function over the allele of the gene.
        :param f: The function to apply to the allele.
        :return: A new gene with the mapped allele.
        """
        return Gene.from_python(self._pyobj.map(f))


"""
Some helper functions for creating gene instances.

This makes it possible to create genes like:
>>> rd.gene.float(0.5)
FloatGene<0.5>
"""


def float(
    allele: float | None = None,
    *,
    init_range: tuple[float, float] | None = None,
    bounds: tuple[float, float] | None = None,
):
    float_gene = PyGene.float(allele=allele, range=init_range, bounds=bounds)
    return Gene.from_python(float_gene)


def int(
    allele: int | None = None,
    *,
    init_range: tuple[int, int] | None = None,
    bounds: tuple[int, int] | None = None,
):
    int_gene = PyGene.int(allele=allele, range=init_range, bounds=bounds)
    return Gene.from_python(int_gene)


def bit(
    allele: bool | None = None,
):
    bit_gene = PyGene.bit(allele=allele)
    return Gene.from_python(bit_gene)


def char(
    allele: str | None = None,
    char_set: set[str] | None = None,
):
    char_gene = PyGene.char(
        allele=allele, char_set=list(char_set) if char_set else None
    )
    return Gene.from_python(char_gene)
