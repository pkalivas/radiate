# from __future__ import annotations

# from collections.abc import Sequence
# from typing import TYPE_CHECKING, Any, override
# from abc import ABC

# from radiate._typing import RdDataType
# from radiate.dtype import Float64, Int64
# from radiate.genome import GeneType

# from .base import ShapedCodec

# if TYPE_CHECKING:
#     from radiate.genome import GeneType


# class NumericCodec[A, T](ShapedCodec[A, T], ABC):
#     def __init__(
#         self,
#         dtype: RdDataType | None = None,
#         shape: int | Sequence[int] = 1,
#         init_range: tuple[A, A] | None = None,
#         bounds: tuple[A, A] | None = None,
#         use_numpy: bool = False,
#     ):
#         super().__init__(shape)
#         self.init_range = init_range
#         self.bounds = bounds
#         self.use_numpy = use_numpy
#         self.encode_dtype = dtype

#     @override
#     @classmethod
#     def scalar(
#         cls,
#         init_range: tuple[A, A] | None = None,
#         bounds: tuple[A, A] | None = None,
#         use_numpy: bool = False,
#         dtype: RdDataType | None = None,
#     ) -> NumericCodec[A, T]:
#         return cls(
#             shape=[1],
#             init_range=init_range,
#             bounds=bounds,
#             use_numpy=use_numpy,
#             dtype=dtype,
#         )

#     @override
#     @classmethod
#     def vector(
#         cls,
#         length: int,
#         init_range: tuple[A, A] | None = None,
#         bounds: tuple[A, A] | None = None,
#         use_numpy: bool = False,
#         dtype: RdDataType | None = None,
#     ) -> NumericCodec[A, T]:
#         return cls(
#             shape=length,
#             init_range=init_range,
#             bounds=bounds,
#             use_numpy=use_numpy,
#             dtype=dtype,
#         )

#     @override
#     @classmethod
#     def matrix(
#         cls,
#         shape: Sequence[int],
#         init_range: tuple[A, A] | None = None,
#         bounds: tuple[A, A] | None = None,
#         use_numpy: bool = False,
#         dtype: RdDataType | None = None,
#     ) -> NumericCodec[A, T]:
#         return cls(
#             shape=shape,
#             init_range=init_range,
#             bounds=bounds,
#             use_numpy=use_numpy,
#             dtype=dtype,
#         )


# class FloatCodec[T](NumericCodec[float, T]):
#     gene_type = GeneType.FLOAT

#     def __init__(
#         self,
#         shape: int | Sequence[int] = 1,
#         init_range: tuple[float, float] | None = None,
#         bounds: tuple[float, float] | None = None,
#         use_numpy: bool = False,
#         dtype: RdDataType | None = None,
#     ):
#         super().__init__(
#             shape=shape,
#             dtype=dtype,
#             init_range=init_range,
#             bounds=bounds,
#             use_numpy=use_numpy,
#         )

#     def __build__backend__(self) -> Any:
#         from radiate.radiate import PyFloatCodec

#         return PyFloatCodec.matrix(
#             chromosome_lengths=self._shape,
#             value_range=self.init_range,
#             bound_range=self.bounds,
#             use_numpy=self.use_numpy,
#             dtype=str(self.encode_dtype)
#             if self.encode_dtype is not None
#             else str(Float64),
#         )


# class IntCodec[T](NumericCodec[int, T]):
#     gene_type = GeneType.INT

#     def __init__(
#         self,
#         shape: int | Sequence[int] = 1,
#         init_range: tuple[int, int] | None = None,
#         bounds: tuple[int, int] | None = None,
#         use_numpy: bool = False,
#         dtype: RdDataType | None = None,
#     ):
#         super().__init__(
#             shape=shape,
#             dtype=dtype,
#             init_range=init_range,
#             bounds=bounds,
#             use_numpy=use_numpy,
#         )

#     def __build__backend__(self) -> Any:
#         from radiate.radiate import PyIntCodec

#         return PyIntCodec.matrix(
#             chromosome_lengths=self._shape,
#             value_range=self.init_range,
#             bound_range=self.bounds,
#             use_numpy=self.use_numpy,
#             dtype=str(self.encode_dtype)
#             if self.encode_dtype is not None
#             else str(Int64),
#         )
