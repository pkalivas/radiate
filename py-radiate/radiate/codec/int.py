from __future__ import annotations

from radiate.genome.chromosome import Chromosome
from radiate.genome.gene import Gene

from .base import CodecBase
from radiate.genome import Genotype, GeneType
from radiate._bridge.wrapper import RsObject
from radiate.radiate import PyIntCodec
from radiate._typing import AtLeastOne, RdDataType
from radiate.dtype import IntegerType, Int64


class IntCodec[D](CodecBase[int, D], RsObject):
    gene_type = GeneType.INT

    def __init__(
        self,
        shape: AtLeastOne[int] | None = None,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        genes: AtLeastOne[Gene[int]] | None = None,
        chromosomes: AtLeastOne[Chromosome[int]] | None = None,
        use_numpy: bool = False,
        dtype: RdDataType | None = None,
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
                init_range=init_range, bounds=bounds, use_numpy=use_numpy, dtype=dtype
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
    def from_genes(
        genes: list[Gene[int]] | tuple[Gene[int], ...], use_numpy: bool = False
    ) -> IntCodec[list[int]]:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new IntCodec instance with the specified genes.
        """
        return IntCodec(
            genes=genes,
            use_numpy=use_numpy,
        )

    @staticmethod
    def from_chromosomes(
        chromosomes: list[Chromosome[int]] | tuple[Chromosome[int], ...],
        use_numpy: bool = False,
    ) -> IntCodec[list[list[int]]]:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new IntCodec instance with the specified chromosomes.
        """
        return IntCodec(chromosomes=chromosomes, use_numpy=use_numpy)

    @staticmethod
    def matrix(
        shape: tuple[int, int] | list[int],
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: bool = False,
        dtype: RdDataType | None = None,
    ) -> IntCodec[list[list[int]]]:
        """
        Initialize the int codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
        :param use_numpy: Whether to use NumPy for the underlying data representation.
        :param dtype: The integer data type to use (e.g., int8, int16, int32, int64).
        :return: A new IntCodec instance with the specified configuration.
        """
        return IntCodec(
            shape=shape,
            init_range=init_range,
            bounds=bounds,
            use_numpy=use_numpy,
            dtype=dtype,
        )

    @staticmethod
    def vector(
        length: int,
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: bool = False,
        dtype: RdDataType | None = None,
    ) -> IntCodec[list[int]]:
        """
        Create a vector codec with specified length.
        :param length: Length of the vector.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
        :return: A new IntCodec instance with vector configuration.
        """
        return IntCodec(
            shape=length,
            init_range=init_range,
            bounds=bounds,
            use_numpy=use_numpy,
            dtype=dtype,
        )

    @staticmethod
    def scalar(
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType | None = None,
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
        genes: AtLeastOne[Gene[int]], use_numpy: bool = False
    ) -> PyIntCodec:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new IntCodec instance with the specified genes.
        """
        from radiate.genome import GeneType

        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")
        if not all(g.gene_type() == GeneType.INT for g in genes):
            raise TypeError("All genes must be of type 'int'.")

        return PyIntCodec.from_genes(
            list(map(lambda g: g.py_gene(), genes)), use_numpy=use_numpy
        )

    @staticmethod
    def __from_chromosomes(
        chromosomes: list[Chromosome[int]] | tuple[Chromosome[int], ...],
    ) -> PyIntCodec:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new PyIntCodec instance with the specified chromosomes.
        """
        from radiate.genome import GeneType

        if not isinstance(chromosomes, (list, tuple)):
            raise TypeError(
                "chromosomes must be a list or tuple of Chromosome instances."
            )
        if not all(
            g.gene_type() == GeneType.INT for c in chromosomes for g in c.genes()
        ):
            raise TypeError("All chromosomes must be of type 'int'.")

        return PyIntCodec.from_chromosomes(
            list(map(lambda c: c.__backend__(), chromosomes))
        )

    @staticmethod
    def __matrix(
        shape: AtLeastOne[int],
        init_range: tuple[int, int] | None = None,
        bounds: tuple[int, int] | None = None,
        use_numpy: bool = False,
        dtype: RdDataType | None = None,
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
            if not isinstance(dtype, RdDataType):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
            if not dtype.is_integer():
                raise ValueError("dtype must be an integer data type.")
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
        dtype: RdDataType | None = None,
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
            if bounds[1] < init_range[0]:
                raise ValueError("Maximum bound must be non-negative.")
        if dtype is not None:
            if not isinstance(dtype, RdDataType):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
            if not dtype.is_integer():
                raise ValueError("dtype must be an integer data type.")
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
        dtype: RdDataType | None = None,
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
            if bounds[1] < init_range[0]:
                raise ValueError("Maximum bound must be non-negative.")

        if dtype is not None:
            if not isinstance(dtype, RdDataType):
                raise ValueError(
                    "dtype must be an instance of DataType or DataTypeClass."
                )
            if not dtype.is_integer():
                raise ValueError("dtype must be an integer data type.")
        else:
            dtype = Int64  # Default to int64 if no dtype is provided

        return PyIntCodec.scalar(
            value_range=init_range,
            bound_range=bounds,
            dtype=str(dtype),
        )
