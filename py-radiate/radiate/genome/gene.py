from __future__ import annotations

from abc import ABC

from typing import Optional, Tuple, Set
from enum import Enum

from radiate.gp.op import Op
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


class Gene[T](ABC):
    def __init__(self, py_gene: PyGene) -> None:
        self.__inner = py_gene

    @classmethod
    def _from_py_gene(cls, py_gene: PyGene) -> Gene[T]:
        """
        Create a Gene instance from a PyGene instance.
        :param py_gene: An instance of PyGene.
        :return: A new Gene instance.
        """
        if not isinstance(py_gene, PyGene):
            raise TypeError("py_gene must be an instance of PyGene.")
        instance = cls.__new__(cls)
        instance.__inner = py_gene
        return instance
    
    @staticmethod
    def from_python(py_gene: PyGene) -> Gene[T]:
        """
        Create a Gene instance from a PyGene instance.
        :param py_gene: An instance of PyGene.
        :return: A new Gene instance.
        """
        match GeneType.from_str(py_gene.gene_type()):
            case GeneType.FLOAT:
                return FloatGene._from_py_gene(py_gene)
            case GeneType.INT:
                return IntGene._from_py_gene(py_gene)
            case GeneType.BIT:
                return BitGene._from_py_gene(py_gene)
            case GeneType.CHAR:
                return CharGene._from_py_gene(py_gene)
            case _:
                raise ValueError(f"Unsupported gene type: {py_gene.gene_type()}")
            
    def to_python(self) -> PyGene:
        """
        Converts the Gene instance to a PyGene instance.
        :return: A PyGene instance.
        """
        return self.__inner

    def __repr__(self) -> str:
        """
        Return a string representation of the Gene instance.
        :return: String representation of the Gene instance.
        """
        return f"{self.__class__.__name__}<{self.__inner.__repr__()}>"

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


class FloatGene(Gene[float]):
    def __init__(
        self,
        allele: Optional[float] = None,
        *,
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
    ) -> None:
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
        super().__init__(
            PyGene.float(allele=allele, range=value_range, bounds=bound_range)
        )


class IntGene(Gene[int]):
    def __init__(
        self,
        allele: Optional[int] = None,
        *,
        value_range: Optional[Tuple[int, int]] = None,
        bound_range: Optional[Tuple[int, int]] = None,
    ) -> None:
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
        super().__init__(
            PyGene.int(allele=allele, range=value_range, bounds=bound_range)
        )


class BitGene(Gene[bool]):
    def __init__(self, allele: Optional[bool] = None) -> None:
        """
        Create a float gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as a float gene.

        Example
        --------
        >>> rd.float(allele=5.0, value_range=(-10.0, 10.0), bound_range=(-20.0, 20.0))
        Gene(5.0)
        """
        super().__init__(PyGene.bit(allele=allele))


class CharGene(Gene[str]):
    def __init__(
        self,
        allele: Optional[str] = None,
        char_set: Optional[Set[str]] = None,
    ) -> None:
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
        super().__init__(
            PyGene.char(allele=allele, char_set=list(char_set) if char_set else None)
        )

class PermutationGene[T](Gene[T]):
    def __init__(self, allele: Optional[T] = None, index: int = 0) -> None:
        """
        Create a permutation gene with optional allele.
        :param allele: Initial value of the gene.
        :return: A new Gene instance configured as a permutation gene.

        Example
        --------
        >>> rd.Gene.permutation(allele=[1, 2, 3])
        Gene([1, 2, 3])
        """
        super().__init__(PyGene.permutation(allele=allele, index=index))

class GraphNodeGene(Gene[Op]):
    def __init__(self, index: int, allele: Op, node_type: str) -> None:
        """
        Create a graph node gene with specified index, allele, and node type.
        :param index: Index of the graph node.
        :param allele: Allele of the graph node.
        :param node_type: Type of the graph node.
        :return: A new Gene instance configured as a graph node gene.
        """
        super().__init__(PyGene.graph_gene(index, allele, node_type))

    # @staticmethod
    # def float(
    #     allele: Optional[float] = None,
    #     *,
    #     value_range: Optional[Tuple[float, float]] = None,
    #     bound_range: Optional[Tuple[float, float]] = None,
    # ) -> Gene[float]:
    #     """
    #     Create a float gene with optional allele, value range, and bound range.
    #     :param allele: Initial value of the gene.
    #     :param value_range: Minimum and maximum value for the gene.
    #     :param bound_range: Minimum and maximum bound for the gene.
    #     :return: A new Gene instance configured as a float gene.

    #     Example
    #     --------
    #     >>> rd.Gene.float(allele=5.0, value_range=(-10.0, 10.0), bound_range=(-20.0, 20.0))
    #     Gene(5.0)
    #     """
    #     return Gene(PyGene.float(allele=allele, range=value_range, bounds=bound_range))

    # @staticmethod
    # def int(
    #     allele: int | None = None,
    #     *,
    #     value_range: Tuple[int, int] | None = None,
    #     bound_range: Tuple[int, int] | None = None,
    # ) -> Gene[int]:
    #     """
    #     Create an integer gene with optional allele, value range, and bound range.
    #     :param allele: Initial value of the gene.
    #     :param value_range: Minimum and maximum value for the gene.
    #     :param bound_range: Minimum and maximum bound for the gene.
    #     :return: A new Gene instance configured as an integer gene.

    #     Example
    #     --------
    #     >>> rd.Gene.int(allele=5, value_range=(0, 10), bound_range=(-5, 15))
    #     Gene(5)
    #     """
    #     return Gene(PyGene.int(allele=allele, range=value_range, bounds=bound_range))

    # @staticmethod
    # def bit(allele: bool | None = None) -> Gene[bool]:
    #     """
    #     Create a bit gene with an optional allele.
    #     :param allele: Initial value of the gene.
    #     :return: A new Gene instance configured as a bit gene.

    #     Example
    #     --------
    #     >>> rd.Gene.bit(allele=True)
    #     Gene(True)
    #     """
    #     return Gene(PyGene.bit(allele=allele))

    # @staticmethod
    # def char(allele: str | None = None, char_set: set[str] | None = None) -> Gene[str]:
    #     """
    #     Create a character gene with optional allele, value range, and bound range.
    #     :param allele: Initial value of the gene.
    #     :param value_range: Minimum and maximum value for the gene.
    #     :param bound_range: Minimum and maximum bound for the gene.
    #     :return: A new Gene instance configured as a character gene.

    #     Example
    #     --------
    #     >>> rd.Gene.char(allele='a', char_set={'a', 'b', 'c'})
    #     Gene(a)
    #     """
    #     return Gene(
    #         PyGene.char(allele=allele, char_set=list(char_set) if char_set else None)
    #     )
