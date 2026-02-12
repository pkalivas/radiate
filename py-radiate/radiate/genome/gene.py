from __future__ import annotations

from radiate.radiate import PyGene
from radiate._bridge.wrapper import RsObject
from radiate.dtype import Float64, Int64
from radiate._typing import RdDataType

from . import GeneType


class Gene[T](RsObject):
    @staticmethod
    def float(
        allele: float | None = None,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType | None = Float64,
    ) -> Gene[float]:
        float_gene = PyGene.float(
            allele=allele, range=init_range, bounds=bounds, dtype=str(dtype)
        )
        return Gene.from_rust(float_gene)

    @staticmethod
    def int(
        allele: int | None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType | None = Int64,
    ) -> Gene[int]:
        int_gene = PyGene.int(
            allele=allele, range=init_range, bounds=bounds, dtype=str(dtype)
        )
        return Gene.from_rust(int_gene)

    @staticmethod
    def bit(allele: bool | None = None) -> Gene[bool]:
        bit_gene = PyGene.bit(allele=allele)
        return Gene.from_rust(bit_gene)

    @staticmethod
    def char(allele: str | None = None, char_set: set[str] | None = None) -> Gene[str]:
        char_gene = PyGene.char(
            allele=allele, char_set=list(char_set) if char_set else None
        )
        return Gene.from_rust(char_gene)

    def __repr__(self):
        return f"{self.gene_type().value}({self.allele()}, dtype={self.dtype()})"

    def gene_type(self) -> "GeneType":
        """
        Get the type of the gene.
        :return: The type of the gene as a string.
        """
        from . import GeneType

        return GeneType.from_str(self.__backend__().gene_type().name())

    def allele(self) -> T:
        """
        Get the allele of the gene.
        :return: The allele of the gene, which can be a float, int, bool, str, or None.
        """
        return self.try_get_cache("allele_value", lambda: self.__backend__().allele())

    def min(self) -> T | None:
        """
        Get the minimum value of the gene, if applicable.
        :return: The minimum value of the gene, or None if not applicable.
        """
        min_value = self.try_get_cache("bounds", lambda: self.__backend__().bounds())
        return min_value[0] if min_value else None

    def max(self) -> T | None:
        """
        Get the maximum value of the gene, if applicable.
        :return: The maximum value of the gene, or None if not applicable.
        """
        max_value = self.try_get_cache("bounds", lambda: self.__backend__().bounds())
        return max_value[1] if max_value else None

    def init_range(self) -> tuple[T, T] | None:
        """
        Get the initial range of the gene, if applicable.
        :return: The initial range of the gene as a tuple (min, max), or None if not applicable.
        """
        return self.try_get_cache("init_range", lambda: self.__backend__().init_range())

    def char_set(self) -> set[str] | None:
        """
        Get the character set of the gene, if applicable.
        :return: The character set of the gene as a set of strings, or None if not applicable.
        """
        char_set = self.try_get_cache("char_set", lambda: self.__backend__().char_set())
        return set(char_set) if char_set else None


# def float(
#     allele: float | None = None,
#     *,
#     init_range: tuple[float, float] | None = None,
#     bounds: tuple[float, float] | None = None,
#     dtype: RdDataType | None = Float64,
# ):
#     float_gene = PyGene.float(
#         allele=allele, range=init_range, bounds=bounds, dtype=str(dtype)
#     )
#     return Gene.from_rust(float_gene)


# def int(
#     allele: int | None = None,
#     *,
#     init_range: tuple[int, int] | None = None,
#     bounds: tuple[int, int] | None = None,
#     dtype: RdDataType | None = Int64,
# ):
#     int_gene = PyGene.int(
#         allele=allele, range=init_range, bounds=bounds, dtype=str(dtype)
#     )
#     return Gene.from_rust(int_gene)


# def bit(allele: bool | None = None):
#     bit_gene = PyGene.bit(allele=allele)
#     return Gene.from_rust(bit_gene)


# def char(
#     allele: str | None = None,
#     char_set: set[str] | None = None,
# ):
#     char_gene = PyGene.char(
#         allele=allele, char_set=list(char_set) if char_set else None
#     )
#     return Gene.from_rust(char_gene)
