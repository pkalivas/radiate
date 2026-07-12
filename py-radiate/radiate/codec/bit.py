from __future__ import annotations

from typing import TYPE_CHECKING, Any, Literal, Sequence, overload

from radiate.radiate import PyBitCodec

from .._bridge import RsObject
from .._typing import AtLeastOne
from ..genome import GeneType, Genotype
from .base import CodecBase

if TYPE_CHECKING:
    from .._dependancies import numpy as np


class BitCodec[D](CodecBase[bool, D], RsObject):
    """BitCodec for bit-based chromosomes. Encodes/decodes to bit strings."""

    @overload
    def __new__(
        cls,
        shape: int,
        *,
        use_numpy: Literal[False] = ...,
    ) -> "BitCodec[list[bool]]": ...

    @overload
    def __new__(
        cls,
        shape: int,
        *,
        use_numpy: Literal[True] = ...,
    ) -> "BitCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        shape: Sequence[int],
        *,
        use_numpy: Literal[False] = ...,
    ) -> "BitCodec[list[list[bool]]]": ...

    @overload
    def __new__(
        cls,
        shape: Sequence[int],
        *,
        use_numpy: Literal[True] = ...,
    ) -> "BitCodec[list[np.ndarray]]": ...

    def __new__(cls, *args: Any, **kwargs: Any) -> "BitCodec[Any]":
        return super().__new__(cls)

    def __init__(
        self,
        shape: AtLeastOne[int] | None = None,
        use_numpy: bool = False,
    ):
        """
        Initialize the bit codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        """
        if shape is not None:
            if isinstance(shape, int):
                self._pyobj = self._vector(length=shape, use_numpy=use_numpy)
            elif isinstance(shape, (tuple, list)):
                self._pyobj = self._matrix(shape=shape, use_numpy=use_numpy)
            else:
                raise ValueError(
                    "Shape must be an int, tuple of ints, or list of ints."
                )
        else:
            raise ValueError("Shape must be provided.")

    def encode(self) -> Genotype[bool]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype[bool]) -> D:
        """
        Decode a Genotype into its bit representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded bit representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.__backend__().decode_py(genotype.__backend__())

    @property
    def gene_type(self) -> GeneType:
        return GeneType.BIT

    @staticmethod
    def _matrix(shape: AtLeastOne[int], use_numpy: bool = False) -> BitCodec[Any]:
        if isinstance(shape, tuple):
            if len(shape) != 2:
                raise ValueError("Shape must be a tuple of (rows, cols).")
            rows, cols = shape
            if rows < 1 or cols < 1:
                raise ValueError("Rows and columns must be at least 1.")
            shape = [cols for _ in range(rows)]
        elif isinstance(shape, list):
            if not all(isinstance(x, int) and x > 0 for x in shape):
                raise ValueError("Shape must be a list of positive integers.")

        return PyBitCodec.matrix(chromosome_lengths=shape, use_numpy=use_numpy)

    @staticmethod
    def _vector(length: int = 8, use_numpy: bool = False) -> BitCodec[Any]:
        return PyBitCodec.vector(chromosome_length=length, use_numpy=use_numpy)
