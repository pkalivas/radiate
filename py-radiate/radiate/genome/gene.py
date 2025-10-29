from __future__ import annotations

from typing import TYPE_CHECKING

from radiate.radiate import PyGene
from radiate.wrapper import PyObject

if TYPE_CHECKING:
    from . import GeneType


class Gene[T](PyObject[PyGene]):
    @classmethod
    def __factory__(cls):
        instance = cls.__new__(cls)
        instance.__init__()
        return instance

    def __repr__(self):
        return f"{self.gene_type().value}({self.allele()})"

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
        return self.__backend__().allele()


class AnyGene(Gene):
    def __backend__(self) -> PyGene:
        if "_pyobj" not in self.__dict__:
            properties = self.__dict__
            metadata = {
                "__class__": f"{self.__class__.__module__}.{self.__class__.__qualname__}"
            }

            self._pyobj = PyGene.any(
                allele=properties,
                metadata=metadata,
                factory=lambda: self.__class__.__factory__().__dict__,
            )

        return self._pyobj

    def allele(self) -> AnyGene:
        return self.__dict__


def float(
    allele: float | None = None,
    *,
    init_range: tuple[float, float] | None = None,
    bounds: tuple[float, float] | None = None,
):
    float_gene = PyGene.float(allele=allele, range=init_range, bounds=bounds)
    return Gene.from_rust(float_gene)


def int(
    allele: int | None = None,
    *,
    init_range: tuple[int, int] | None = None,
    bounds: tuple[int, int] | None = None,
):
    int_gene = PyGene.int(allele=allele, range=init_range, bounds=bounds)
    return Gene.from_rust(int_gene)


def bit(
    allele: bool | None = None,
):
    bit_gene = PyGene.bit(allele=allele)
    return Gene.from_rust(bit_gene)


def char(
    allele: str | None = None,
    char_set: set[str] | None = None,
):
    char_gene = PyGene.char(
        allele=allele, char_set=list(char_set) if char_set else None
    )
    return Gene.from_rust(char_gene)
