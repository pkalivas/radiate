from typing import Callable

from radiate.genome.genotype import Genotype
from radiate.genome.gene import AnyGene
from radiate.radiate import PyAnyCodec

from . import CodecBase


class AnyCodec[T: AnyGene](CodecBase[T, list[T]]):
    def __init__(self, len: int, genes_factory: Callable[[], T]):
        """
        Initialize the AnyCodec with encoder and decoder functions.
        :param len: The number of genes in the codec.
        :param genes_factory: A callable that produces new gene instances.
        """
        values = [genes_factory() for _ in range(len)]

        factories = {
            g.__class__.__module__
            + "."
            + g.__class__.__qualname__: g.__class__.__from_gene__
            for g in values
        }

        def creator(gene_dict):
            cls_name = gene_dict.get("__class__")
            if cls_name is None:
                raise ValueError("Gene dictionary must contain a '__class__' key.")
            body = {k: v for k, v in gene_dict.items() if k != "__class__"}
            try:
                return factories[cls_name](body)
            except KeyError:
                raise ValueError(f"Unknown class '{cls_name}' in gene dictionary.")

        self.codec = PyAnyCodec(
            list(map(lambda g: g.__to_gene__(), values)),
            creator,
            new_instance=lambda: genes_factory().__to_gene__(),
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
        return self.codec.decode_py(genotype.backend())
