from __future__ import annotations

from typing import Sequence, overload, Any, Literal, TYPE_CHECKING

from .base import CodecBase

from radiate.radiate import PyFloatCodec
from radiate.genome import Genotype, Gene, Chromosome, GeneType
from radiate.dtype import DataType, DataTypeClass, Float64
from radiate._bridge.wrapper import RsObject
from radiate._typing import (
    AtLeastOne,
)

if TYPE_CHECKING:
    from radiate._dependancies import numpy as np


class FloatCodec[D](CodecBase[float, D], RsObject):
    gene_type = GeneType.FLOAT

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: None = ...,
        use_numpy: bool = ...,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[float]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: Gene[float] = ...,
        chromosomes: None = ...,
        use_numpy: bool = ...,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[float]": ...

    @overload
    def __new__(
        cls,
        shape: int,
        *,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: None = ...,
        use_numpy: Literal[False] = False,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[list[float]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: Sequence[Gene[float]] = ...,
        chromosomes: None = ...,
        use_numpy: Literal[False] = False,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[list[float]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: Sequence[Gene[float]] = ...,
        chromosomes: None = ...,
        use_numpy: Literal[True] = True,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: Chromosome[float] = ...,
        use_numpy: Literal[True] = True,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: Chromosome[float] = ...,
        use_numpy: Literal[False] = False,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[list[float]]": ...

    @overload
    def __new__(
        cls,
        shape: int,
        *,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: None = ...,
        use_numpy: Literal[True] = True,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        shape: Sequence[int],
        *,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: None = ...,
        use_numpy: Literal[False] = ...,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[list[list[float]]]": ...

    @overload
    def __new__(
        cls,
        shape: Sequence[int],
        *,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: None = ...,
        use_numpy: Literal[True] = ...,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[list[np.ndarray]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[float]],
        use_numpy: Literal[False] = ...,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[list[list[float]]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = ...,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[float]],
        use_numpy: Literal[True] = ...,
        dtype: DataTypeClass | DataType | None = ...,
    ) -> "FloatCodec[list[np.ndarray]]": ...

    def __new__(cls, *args: Any, **kwargs: Any) -> "FloatCodec[Any]":
        return super().__new__(cls)

    def __init__(
        self,
        shape: AtLeastOne[int] | None = None,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: Gene[float] | Sequence[Gene[float]] | None = None,
        chromosomes: AtLeastOne[Chromosome[float]] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ):
        """
        Initialize the float codec with number of chromosomes and value bounds.
        :param shape: Number of chromosomes with the number of genes in each chromosome.
        :param init_range: Range for initializing gene values.
        :param bounds: Bounds for gene values.
        """
        provided = sum(x is not None for x in [shape, genes, chromosomes])

        if provided > 1:
            raise ValueError(
                "Only one of shape, genes, or chromosomes should be provided."
            )

        if shape is not None:
            if isinstance(shape, int):
                self._pyobj = self.__vector(
                    length=shape,
                    init_range=init_range,
                    bounds=bounds,
                    use_numpy=use_numpy,
                    dtype=dtype,
                )
            elif isinstance(shape, (tuple, list)):
                self._pyobj = self.__matrix(
                    shape=shape,
                    init_range=init_range,
                    bounds=bounds,
                    use_numpy=use_numpy,
                    dtype=dtype,
                )
            else:
                raise TypeError("shape must be an int, tuple[int, int], or list[int].")
        elif genes is not None:
            self._pyobj = self.__from_genes(genes=genes, use_numpy=use_numpy)
        elif chromosomes is not None:
            self._pyobj = self.__from_chromosomes(
                chromosomes=chromosomes, use_numpy=use_numpy
            )
        else:
            self._pyobj = self.__scalar(
                init_range=init_range,
                bounds=bounds,
                use_numpy=use_numpy,
                dtype=dtype,
            )

    def encode(self) -> Genotype[float]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype[float]) -> D:
        """
        Decode a Genotype into its float representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded float representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.__backend__().decode_py(genotype=genotype.__backend__())

    @staticmethod
    def __scalar(
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ) -> PyFloatCodec:
        if init_range is not None:
            if len(init_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if init_range[0] >= init_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
        if bounds is not None:
            if len(bounds) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bounds[0] >= bounds[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")

        if dtype is not None:
            if not isinstance(dtype, (DataType, DataTypeClass)):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
        else:
            dtype = Float64  # Default to float64 if no dtype is provided

        return PyFloatCodec.scalar(
            value_range=init_range,
            bound_range=bounds,
            use_numpy=use_numpy,
            dtype=str(dtype),
        )

    @staticmethod
    def __vector(
        length: int,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ) -> PyFloatCodec:
        if length <= 0:
            raise ValueError("Length must be a positive integer.")

        if init_range is not None:
            if len(init_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if init_range[0] >= init_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
        if bounds is not None:
            if len(bounds) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bounds[0] >= bounds[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")

        if dtype is not None:
            if not isinstance(dtype, (DataType, DataTypeClass)):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
        else:
            dtype = Float64  # Default to float64 if no dtype is provided

        return PyFloatCodec.vector(
            length=length,
            value_range=init_range,
            bound_range=bounds,
            use_numpy=use_numpy,
            dtype=str(dtype),
        )

    @staticmethod
    def __matrix(
        shape: Sequence[int],
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ) -> PyFloatCodec:
        shapes = None
        if isinstance(shape, tuple):
            if len(shape) != 2:
                raise ValueError("Shape must be a tuple of (rows, cols).")
            rows, cols = shape
            shapes = [cols for _ in range(rows)]
        elif isinstance(shape, list):
            shapes = shape

        if init_range is not None:
            if len(init_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if init_range[0] >= init_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
        if bounds is not None:
            if len(bounds) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bounds[0] >= bounds[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")

        if dtype is not None:
            if not isinstance(dtype, (DataType, DataTypeClass)):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
        else:
            dtype = Float64  # Default to float64 if no dtype is provided

        return PyFloatCodec.matrix(
            chromosome_lengths=shapes,
            value_range=init_range,
            bound_range=bounds,
            use_numpy=use_numpy,
            dtype=str(dtype),
        )

    @staticmethod
    def __from_genes(
        genes: Gene[float] | Sequence[Gene[float]] | None = None,
        use_numpy: bool = False,
    ) -> PyFloatCodec:
        if isinstance(genes, Gene):
            genes = [genes]
        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")
        if isinstance(genes, Gene):
            genes = [genes]
        return PyFloatCodec.from_genes(
            list(map(lambda g: g.__backend__(), genes)), use_numpy=use_numpy
        )

    @staticmethod
    def __from_chromosomes(
        chromosomes: AtLeastOne[Chromosome[float]],
        use_numpy: bool = False,
    ) -> PyFloatCodec:
        from radiate.genome import GeneType

        if isinstance(chromosomes, Chromosome):
            chromosomes = [chromosomes]
        if not isinstance(chromosomes, (list, tuple)):
            raise TypeError(
                "chromosomes must be a list or tuple of Chromosome instances."
            )
        if not all(g.gene_type() == GeneType.FLOAT for c in chromosomes for g in c):
            raise TypeError("All chromosomes must be of type 'float'.")

        return PyFloatCodec.from_chromosomes(
            list(map(lambda c: c.__backend__(), chromosomes)), use_numpy=use_numpy
        )
