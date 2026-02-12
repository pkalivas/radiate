from __future__ import annotations

from collections.abc import Sequence
from typing import TYPE_CHECKING
from abc import ABC, abstractmethod

from radiate.genome import Gene

if TYPE_CHECKING:
    from radiate.genome import Genotype, GeneType
    # from radiate._typing import Encoding


class CodecBase[T, D](ABC):
    gene_type: "GeneType"

    @abstractmethod
    def encode(self) -> "Genotype[T]":
        raise NotImplementedError("encode method must be implemented by subclasses.")

    @abstractmethod
    def decode(self, genotype: "Genotype[T]") -> D:
        raise NotImplementedError("decode method must be implemented by subclasses.")

    @abstractmethod
    def from_genes(
        genes: Gene[T] | Sequence[Gene[T]] | Sequence[Sequence[Gene[T]]],
        use_numpy: bool = False,
    ) -> "CodecBase[T, D] | None":
        return None


# type Temp = "Gene" | Sequence["Gene"] | Sequence[Sequence["Gene"]] | "CodecBase"


# def extract_codec[A, T](encoding: Temp) -> CodecBase[A, T]:
#     from radiate.genome import Gene
#     from .bit import BitCodec
#     from .char import CharCodec
#     from .float import FloatCodec
#     from .int import IntCodec

#     if isinstance(encoding, CodecBase):
#         return encoding
#     elif isinstance(encoding, Gene):
#         match encoding.gene_type():
#             case GeneType.FLOAT:
#                 return FloatCodec.from_gene(encoding)
#             case GeneType.INT:
#                 return IntCodec.from_gene(encoding)
#             case GeneType.BIT:
#                 return BitCodec.from_gene(encoding)
#             case GeneType.CHAR:
#                 return CharCodec.from_gene(encoding)
#             case _:
#                 raise TypeError(f"Unsupported gene type: {encoding.gene_type()}")
#     #     return encoding.codec()
#     # elif isinstance(encoding, Sequence) and all(isinstance(g, Gene) for g in encoding):
#     #     return encoding[0].codec()
#     # elif (
#     #     isinstance(encoding, Sequence)
#     #     and all(isinstance(seq, Sequence) for seq in encoding)
#     #     and all(isinstance(g, Gene) for seq in encoding for g in seq)
#     # ):
#     #     return encoding[0][0].codec()
#     # else:
#     #     raise TypeError(
#     #         "Encoding must be a CodecBase instance, a Gene, a sequence of Genes, or a sequence of sequences of Genes."
#     #     )
