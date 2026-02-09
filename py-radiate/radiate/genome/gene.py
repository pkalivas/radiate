from __future__ import annotations

from typing import TYPE_CHECKING, Any

from radiate.radiate import PyGene
from radiate.wrapper import PyObject
from radiate.dtype import DataType, dtype_from_str, Float64, Int64, DataTypeClass

if TYPE_CHECKING:
    from . import GeneType


class Gene[T](PyObject[PyGene]):
    @classmethod
    def __factory__(cls):
        instance = cls.__new__(cls)
        instance.__init__()
        return instance

    def __repr__(self):
        return f"{self.gene_type().value}({self.allele()}, dtype={self.dtype()})"

    def gene_type(self) -> "GeneType":
        """
        Get the type of the gene.
        :return: The type of the gene as a string.
        """
        from . import GeneType

        return GeneType.from_str(self.__backend__().gene_type().name())

    def dtype(self) -> DataType | None:
        """
        Get the data type of the gene, if applicable.
        :return: The data type of the gene as a string, or None if not applicable.
        """
        dtype_str = self.__backend__().dtype()
        return dtype_from_str(dtype_str) if dtype_str else None

    def allele(self) -> T:
        """
        Get the allele of the gene.
        :return: The allele of the gene, which can be a float, int, bool, str, or None.
        """
        return self.try_get_cache("allele_value", lambda: self.__backend__().allele())


class AnyGene(Gene[dict[str, Any]]):
    @staticmethod
    def from_json(json_str: str) -> "AnyGene":
        """
        Deserialize a JSON string to an AnyGene object.
        :param json_str: The JSON string representation of the AnyGene.
        :return: An AnyGene object.
        """

        import json

        re_initializing = json.loads(json_str)
        class_name = re_initializing.get("__class__")
        if class_name:
            module_name, _, class_name = class_name.rpartition(".")
            module = __import__(module_name, fromlist=[class_name])
            cls = getattr(module, class_name)
            re_initializing = cls.__factory__()  # type: ignore
            re_initializing.__dict__.update(json.loads(json_str) )
            return re_initializing
        else:
            raise ValueError(
                "JSON string does not contain '__class__' information for deserialization."
            )

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

    def allele(self) -> dict[str, Any]:
        return self.__dict__

    def to_json(self) -> str:
        """
        Serialize the gene to a JSON string.
        :return: A JSON string representation of the gene.
        """
        import json

        temp = json.dumps(
            self.allele()
            | {
                "__class__": f"{self.__class__.__module__}.{self.__class__.__qualname__}"
            }
        )

        return temp


def float(
    allele: float | None = None,
    *,
    init_range: tuple[float, float] | None = None,
    bounds: tuple[float, float] | None = None,
    dtype: DataType | DataTypeClass | None = Float64,
):
    float_gene = PyGene.float(
        allele=allele, range=init_range, bounds=bounds, dtype=str(dtype)
    )
    return Gene.from_rust(float_gene)


def int(
    allele: int | None = None,
    *,
    init_range: tuple[int, int] | None = None,
    bounds: tuple[int, int] | None = None,
    dtype: DataType | DataTypeClass | None = Int64,
):
    int_gene = PyGene.int(
        allele=allele, range=init_range, bounds=bounds, dtype=str(dtype)
    )
    return Gene.from_rust(int_gene)


def bit(allele: bool | None = None):
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
