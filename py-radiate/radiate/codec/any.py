from typing import Callable

from radiate.genome.genotype import Genotype
from . import CodecBase
from radiate.radiate import PyAnyCodec


class AnyCodec[T](CodecBase[T, list[T]]):
    def __init__(self, len: int, genes_factory: Callable[[], T]):
        """
        Initialize the AnyCodec with encoder and decoder functions.
        :param encoder: A callable that encodes a value.
        :param decoder: A callable that decodes a value.
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
        return Genotype.from_python(self.codec.encode_py())

    def decode(self, genotype: Genotype) -> T:
        """
        Decodes a PyAnyCodec into its representation.
        :param genotype: A PyAnyCodec instance to decode.
        :return: The decoded representation of the PyAnyCodec.
        """
        genotype = genotype.to_python()
        return self.codec.decode_py(genotype)
