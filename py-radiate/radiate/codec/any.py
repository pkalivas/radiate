from __future__ import annotations
from typing import Iterable

from radiate.genome.genotype import Genotype
from radiate.genome.gene import AnyGene
from radiate.genome import GeneType
from radiate.wrapper import RsObject
from radiate.radiate import PyAnyCodec

from . import CodecBase

from typing import Any, Iterable

from radiate.radiate import PyFieldCodec, PyFieldSpec

# from radiate.wrapper import RsObject
from radiate.dtype import Field
from radiate._typing import RdDataType


class FieldSpec(RsObject[PyFieldSpec]):
    @classmethod
    def scalar(
        cls,
        *,
        name: str,
        dtype: RdDataType,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        chars: list[str] | None = None,
        choices: list[Any] | None = None,
    ) -> "FieldSpec":
        py = PyFieldSpec(
            Field(name, dtype),
            init_range,
            bounds,
            list(chars) if chars else None,
            choices,
        )
        
        return cls.from_rust(py)

    @classmethod
    def list(cls, *, len: int, inner: "FieldSpec") -> "FieldSpec":
        return cls.from_rust(PyFieldSpec.list(len, inner.__backend__()))

    @classmethod
    def struct(cls, fields: Iterable["FieldSpec"]) -> "FieldSpec":
        return cls.from_rust(PyFieldSpec.struct_([f.__backend__() for f in fields]))


class FieldCodec(RsObject[PyFieldCodec]):
    @classmethod
    def __factory__(cls):
        inst = cls.__new__(cls)
        inst.__init__()
        return inst

    def __init__(self, count: int, specs: list[FieldSpec]):
        self._pyobj = PyFieldCodec(
            count, PyFieldSpec.struct_([s.__backend__() for s in specs])
        )

    def encode(self):
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype):
        # genotype is your PyGenotype wrapper
        return self.__backend__().decode_py(genotype.__backend__())


class AnyCodec[T: AnyGene](CodecBase[T, list[T]], RsObject[PyAnyCodec]):
    gene_type = GeneType.ANY

    def __init__(self, genes: list[T] | Iterable[T]):
        """
        Initialize the AnyCodec with encoder and decoder functions.
        :param len: The number of genes in the codec.
        :param genes_factory: A callable that produces new gene instances.
        """
        if isinstance(genes, list):
            values = genes
        else:
            values = list(genes)

        self._factories = {
            f"{g.__class__.__module__}.{g.__class__.__qualname__}": g.__class__.from_rust
            for g in values
        }

        def creator(gene_dict: dict, metadata: dict):
            cls_name = metadata.get("__class__")
            fn = self._factories.get(cls_name)
            if fn is None:
                raise ValueError(f"Unknown class '{cls_name}'")
            return fn(gene_dict)

        self._pyobj = PyAnyCodec(
            list(map(lambda g: g.__backend__(), values)),
            creator,
        )

    def encode(self) -> Genotype[T]:
        """
        Encodes the codec into a PyAnyCodec.
        :return: A PyAnyCodec instance.
        """
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype) -> list[T]:
        """
        Decodes a PyAnyCodec into its representation.
        :param genotype: A PyAnyCodec instance to decode.
        :return: The decoded representation of the PyAnyCodec.
        """
        return self.__backend__().decode_py(genotype.__backend__())
