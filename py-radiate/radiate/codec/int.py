from __future__ import annotations
from typing import Sequence, overload, Any, Literal, TYPE_CHECKING

from radiate.genome.chromosome import Chromosome
from radiate.genome.gene import Gene

from .base import CodecBase
from radiate.genome import Genotype, GeneType
from radiate._bridge.wrapper import RsObject
from radiate.radiate import PyIntCodec
from radiate._typing import (
    AtLeastOne,
)
from radiate.dtype import DataTypeClass, DataType, Int64

if TYPE_CHECKING:
    from radiate._dependancies import numpy as np


class IntCodec[D](CodecBase[int, D], RsObject):
    gene_type = GeneType.INT

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: bool = ...,
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "IntCodec[int]": ...

    @overload
    def __new__(
        cls,
        shape: int,
        *,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[False] = ...,
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "IntCodec[list[int]]": ...

    @overload
    def __new__(
        cls,
        shape: int,
        *,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[True],
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "IntCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[False] = ...,
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "IntCodec[list[list[int]]]": ...

    @overload
    def __new__(
        cls,
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[True] = ...,
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "IntCodec[list[np.ndarray]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: bool = ...,
        dtype: DataTypeClass | DataType | None = ...,
        genes: Gene[int],
        chromosomes: None = ...,
    ) -> "IntCodec[int]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[False] = ...,
        dtype: DataTypeClass | DataType | None = ...,
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "IntCodec[list[int]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[True],
        dtype: DataTypeClass | DataType | None = ...,
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "IntCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[False],
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: Chromosome[int],
    ) -> "IntCodec[list[int]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[True],
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: Chromosome[int],
    ) -> "IntCodec[np.ndarray]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[False],
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[int]],
    ) -> "IntCodec[list[list[int]]]": ...

    @overload
    def __new__(
        cls,
        *,
        shape: None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: Literal[True],
        dtype: DataTypeClass | DataType | None = ...,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[int]],
    ) -> "IntCodec[list[np.ndarray]]": ...

    def __new__(cls, *args: Any, **kwargs: Any) -> "IntCodec[Any]":
        return super().__new__(cls)

    def __init__(
        self,
        shape: AtLeastOne[int] | None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        genes: Gene[int] | Sequence[Gene[int]] | None = None,
        chromosomes: Chromosome[int] | Sequence[Chromosome[int]] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ):
        """
        Initialize the int codec with a PyIntCodec instance.
        :param codec: An instance of PyIntCodec.
        """
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
                raise ValueError(
                    "Shape must be an int, tuple of ints, or list of ints."
                )
        elif genes is not None:
            self._pyobj = self.__from_genes(genes=genes, use_numpy=use_numpy)
        elif chromosomes is not None:
            self._pyobj = self.__from_chromosomes(
                chromosomes=chromosomes, use_numpy=use_numpy
            )
        elif shape is None and genes is None and chromosomes is None:
            self._pyobj = self.__scalar(
                init_range=init_range, bounds=bounds, dtype=dtype
            )
        else:
            raise ValueError("Shape must be provided.")

    def encode(self) -> Genotype[int]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype[int]) -> D:
        """
        Decode a Genotype into its integer representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded integer representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.__backend__().decode_py(genotype=genotype.__backend__())

    @staticmethod
    def scalar(
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        dtype: DataTypeClass | DataType | None = None,
    ) -> IntCodec[int]:
        """
        Create a scalar codec with specified value and bound ranges.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new IntCodec instance with scalar configuration.
        """
        return IntCodec(
            init_range=init_range,
            bounds=bounds,
            dtype=dtype,
        )

    @staticmethod
    def __from_genes(
        genes: Gene[int] | Sequence[Gene[int]], use_numpy: bool = False
    ) -> PyIntCodec:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new IntCodec instance with the specified genes.
        """
        from radiate.genome import GeneType

        if isinstance(genes, Gene):
            genes = [genes]
        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")
        if not all(g.gene_type() == GeneType.INT for g in genes):
            raise TypeError("All genes must be of type 'int'.")

        return PyIntCodec.from_genes(
            list(map(lambda g: g.__backend__(), genes)), use_numpy=use_numpy
        )

    @staticmethod
    def __from_chromosomes(
        chromosomes: Chromosome[int] | Sequence[Chromosome[int]],
        use_numpy: bool = False,
    ) -> PyIntCodec:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A single Chromosome instance or a sequence of Chromosome instances.
        Returns:
            A new PyIntCodec instance with the specified chromosomes.
        """
        from radiate.genome import GeneType

        if isinstance(chromosomes, Chromosome):
            chromosomes = [chromosomes]
        if not all(
            g.gene_type() == GeneType.INT for c in chromosomes for g in c.genes()
        ):
            raise TypeError("All chromosomes must be of type 'int'.")

        return PyIntCodec.from_chromosomes(
            list(map(lambda c: c.__backend__(), chromosomes)), use_numpy=use_numpy
        )

    @staticmethod
    def __matrix(
        shape: AtLeastOne[int],
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ) -> PyIntCodec:
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
            if not isinstance(dtype, (DataTypeClass, DataType)):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
        else:
            dtype = Int64  # Default to int64 if no dtype is provided

        return PyIntCodec.matrix(
            chromosome_lengths=shapes,
            value_range=init_range,
            bound_range=bounds,
            use_numpy=use_numpy,
            dtype=str(dtype),
        )

    @staticmethod
    def __vector(
        length: int,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ) -> PyIntCodec:
        if length <= 0:
            raise ValueError("Length must be a positive integer.")
        if init_range is not None:
            if len(init_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if init_range[0] >= init_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
            if init_range[1] < init_range[0]:
                raise ValueError("Maximum value must be non-negative.")
        if bounds is not None:
            if len(bounds) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bounds[0] >= bounds[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")
            if bounds[1] < bounds[0]:
                raise ValueError("Maximum bound must be non-negative.")
        if dtype is not None:
            if not isinstance(dtype, (DataTypeClass, DataType)):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
        else:
            dtype = Int64  # Default to int64 if no dtype is provided

        return PyIntCodec.vector(
            length=length,
            value_range=init_range,
            bound_range=bounds,
            use_numpy=use_numpy,
            dtype=str(dtype),
        )

    @staticmethod
    def __scalar(
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        dtype: DataTypeClass | DataType | None = None,
    ) -> PyIntCodec:
        if init_range is not None:
            if len(init_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if init_range[0] >= init_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
            if init_range[1] < init_range[0]:
                raise ValueError("Maximum value must be non-negative.")

        if bounds is not None:
            if len(bounds) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bounds[0] >= bounds[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")
            if bounds[1] < bounds[0]:
                raise ValueError("Maximum bound must be non-negative.")

        if dtype is not None:
            if not isinstance(dtype, (DataTypeClass, DataType)):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
        else:
            dtype = Int64  # Default to int64 if no dtype is provided

        return PyIntCodec.scalar(
            value_range=init_range,
            bound_range=bounds,
            dtype=str(dtype),
        )
