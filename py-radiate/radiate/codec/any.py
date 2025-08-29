from typing import Any, Callable

from radiate.genome.genotype import Genotype
from . import CodecBase
from radiate.radiate import PyAnyCodec


def any_gene(
    _cls=None,
    *,
    factory: Callable[[], Any] | None = None,
    strict_from_gene: bool = False,
):
    """
    Decorator that equips a class with:
      - __to_gene__ / __from_gene__
      - a permanent __factory__ used to create brand-new instances
      - TestGene.new_instance()  (classmethod)
      - obj.new_instance()       (instance method)
    If no factory is supplied, we require a zero-arg constructor.
    """

    def decorate(cls):
        # to/from gene
        def __to_gene__(self):
            return self.__dict__ | {
                "__class__": f"{self.__class__.__module__}.{self.__class__.__qualname__}"
            }

        @classmethod
        def __from_gene__(klass, gene_dict: dict, *, strict: bool = strict_from_gene):
            inst = klass.__new__(klass)  # fresh instance
            if strict:
                # optional: only accept known fields
                unknown = set(gene_dict.keys()) - set(inst.__dict__.keys())
                if unknown:
                    raise ValueError(
                        f"Unknown field(s) {sorted(unknown)} for {klass.__name__}"
                    )
            inst.__dict__.update(gene_dict)
            return inst

        cls.__to_gene__ = __to_gene__
        cls.__from_gene__ = __from_gene__

        return cls

    return decorate if _cls is None else decorate(_cls)


class AnyCodec[T](CodecBase[T, T]):
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
