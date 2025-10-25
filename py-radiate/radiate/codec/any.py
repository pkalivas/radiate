from typing import Iterable

from radiate.genome.genotype import Genotype
from radiate.genome.gene import AnyGene
from radiate.radiate import PyAnyCodec

from . import CodecBase


class AnyCodec[T: AnyGene](CodecBase[T, list[T]]):
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

        self.codec = PyAnyCodec(
            list(map(lambda g: g.__backend__(), values)),
            creator,
        )

    def encode(self) -> Genotype[T]:
        """
        Encodes the codec into a PyAnyCodec.
        :return: A PyAnyCodec instance.
        """
        return Genotype.from_rust(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> list[T]:
        """
        Decodes a PyAnyCodec into its representation.
        :param genotype: A PyAnyCodec instance to decode.
        :return: The decoded representation of the PyAnyCodec.
        """
        return self.codec.decode_py(genotype.__backend__())
