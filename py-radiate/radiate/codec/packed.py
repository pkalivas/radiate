from __future__ import annotations

from typing import TYPE_CHECKING, Any, Literal, overload

from radiate.radiate import PyPackedBitCodec

from .._bridge import RsObject
from ..genome import GeneType, Genotype
from .base import CodecBase

if TYPE_CHECKING:
    from .._dependancies import numpy as np


class PackedBitCodec[D](CodecBase[bool, D], RsObject):
    """
    Codec for a single bit string stored 64 bits per word.

    Use it for long bit strings (roughly 10k bits and up). By default it decodes to
    bits exactly like `BitCodec`, so the same fitness function works with either and
    the gain is memory. With `words=True` it decodes to the raw 64-bit words instead,
    which lets the fitness function work 64 bits at a time. Pair it with
    `use_numpy=True` for speed, e.g. `np.bitwise_count(words).sum()`.
    """

    @overload
    def __new__(
        cls,
        num_bits: int,
        *,
        use_numpy: Literal[False] = ...,
        words: Literal[False] = ...,
    ) -> "PackedBitCodec[list[bool]]": ...

    @overload
    def __new__(
        cls,
        num_bits: int,
        *,
        use_numpy: Literal[True],
        words: Literal[False] = ...,
    ) -> "PackedBitCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        num_bits: int,
        *,
        use_numpy: Literal[False] = ...,
        words: Literal[True],
    ) -> "PackedBitCodec[list[int]]": ...

    @overload
    def __new__(
        cls,
        num_bits: int,
        *,
        use_numpy: Literal[True],
        words: Literal[True],
    ) -> "PackedBitCodec[np.ndarray]": ...

    def __new__(cls, *args: Any, **kwargs: Any) -> "PackedBitCodec[Any]":
        return super().__new__(cls)

    def __init__(
        self, num_bits: int, *, use_numpy: bool = False, words: bool = False
    ) -> None:
        """
        :param num_bits: Number of bits in the bit string. Must be a positive integer.
        :param use_numpy: Decode to a numpy array (`np.bool_`, or `np.uint64` with
            `words=True`) instead of a Python list.
        :param words: Decode to the packed 64-bit words instead of bits. Bit `i` is
            bit `i % 64` of word `i // 64` (least-significant first), and the unused
            bits past `num_bits` in the last word are zero.
        """
        if isinstance(num_bits, bool) or not isinstance(num_bits, int) or num_bits <= 0:
            raise ValueError(
                f"num_bits must be a positive integer for PackedBitCodec, got {num_bits!r}"
            )

        self._pyobj = PyPackedBitCodec.vector(num_bits, use_numpy, words)

    def encode(self) -> Genotype[bool]:
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype[bool]) -> D:
        if not isinstance(genotype, Genotype):
            raise TypeError("Expected a Genotype instance")
        return self.__backend__().decode_py(genotype.__backend__())

    @property
    def gene_type(self) -> GeneType:
        return GeneType.PACKED_BIT
