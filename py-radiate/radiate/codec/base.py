from __future__ import annotations

# from collections.abc import Sequence
from typing import TYPE_CHECKING
from abc import ABC, abstractmethod

from radiate._bridge.wrapper import RsObject

# from radiate.genome import Gene

if TYPE_CHECKING:
    from radiate.genome import Genotype, GeneType
    # from radiate._typing import Encoding


class CodecBase[T, D](RsObject, ABC):
    gene_type: "GeneType"

    @abstractmethod
    def encode(self) -> "Genotype[T]":
        raise NotImplementedError("encode method must be implemented by subclasses.")

    @abstractmethod
    def decode(self, genotype: "Genotype[T]") -> D:
        raise NotImplementedError("decode method must be implemented by subclasses.")

    # @abstractmethod
    # def from_genes(
    #     genes: Gene[T] | Sequence[Gene[T]] | Sequence[Sequence[Gene[T]]],
    #     use_numpy: bool = False,
    # ) -> "CodecBase[T, D] | None":
    #     return None


# from __future__ import annotations

# from collections.abc import Sequence
# from typing import TYPE_CHECKING, Any, Self
# from abc import ABC, abstractmethod

# from radiate._bridge import RsObject
# from radiate.genome import Genotype, GeneType

# if TYPE_CHECKING:
#     from radiate.genome import Genotype, GeneType


# class CodecBase[A, T](RsObject, ABC):
#     gene_type: "GeneType"

#     def __init__(self, shape: int | Sequence[int]):
#         super().__init__()
#         self._shape = self._validate_shape(shape)

#     def __backend__(self) -> Any:
#         if self._pyobj is None:
#             self._pyobj = self.__build__backend__()
#         return self._pyobj

#     @abstractmethod
#     def __build__backend__(self) -> Any:
#         """Build the backend object. Must be implemented by subclasses."""
#         raise NotImplementedError("Subclasses must implement __build__backend__()")

#     def encode(self) -> Genotype[A]:
#         return Genotype.from_rust(self.__backend__().encode_py())

#     def decode(self, genotype: Genotype[A]) -> T:
#         if not isinstance(genotype, Genotype):
#             raise TypeError("genotype must be an instance of Genotype.")
#         return self.__backend__().decode_py(genotype.__backend__())

#     def _validate_shape(self, shape: int | Sequence[int]) -> list[int]:
#         if isinstance(shape, int):
#             if shape <= 0:
#                 raise ValueError("Shape must be a positive integer.")
#             return [shape]
#         elif isinstance(shape, Sequence):
#             if not all(isinstance(dim, int) and dim > 0 for dim in shape):
#                 raise ValueError("All dimensions in shape must be positive integers.")
#             return list(shape)
#         else:
#             return [1]


# class ShapedCodec[A, T](CodecBase[A, T], ABC):
#     def __init__(
#         self,
#         shape: int | Sequence[int],
#     ):
#         super().__init__(shape)

#     @classmethod
#     def scalar(cls, **kwargs) -> Self:
#         return cls(shape=[1], **kwargs)

#     @classmethod
#     def vector(cls, length: int, **kwargs) -> Self:
#         return cls(shape=length, **kwargs)

#     @classmethod
#     def matrix(cls, shape: Sequence[int], **kwargs) -> Self:
#         return cls(shape=shape, **kwargs)
