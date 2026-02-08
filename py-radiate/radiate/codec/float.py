from __future__ import annotations

from .base import CodecBase

from radiate.genome import Genotype, Gene, Chromosome
from radiate.wrapper import PyObject
from radiate.dtype import DataType, DataTypeClass, Float64, FloatType

from radiate.radiate import PyFloatCodec


class FloatCodec[T](CodecBase[float, T], PyObject[PyFloatCodec]):
    def __init__(
        self,
        shape: int | tuple[int, int] | list[int] | None = None,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        genes: Gene[float] | list[Gene[float]] | tuple[Gene[float], ...] | None = None,
        chromosomes: Chromosome[float]
        | list[Chromosome[float]]
        | tuple[Chromosome[float], ...]
        | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ):
        """
        Initialize the float codec with number of chromosomes and value bounds.
        :param shape: Number of chromosomes with the number of genes in each chromosome.
        :param init_range: Range for initializing gene values.
        :param bounds: Bounds for gene values.
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

    def encode(self) -> Genotype[float]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype[float]) -> T:
        """
        Decode a Genotype into its float representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded float representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.__backend__().decode_py(genotype=genotype.__backend__())

    @staticmethod
    def from_genes(
        genes: list[Gene[float]] | tuple[Gene[float], ...], use_numpy: bool = False
    ) -> FloatCodec[list[float]]:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new FloatCodec instance with the specified genes.
        """
        return FloatCodec(
            genes=genes,
            use_numpy=use_numpy,
        )

    @staticmethod
    def from_chromosomes(
        chromosomes: list[Chromosome[float]] | tuple[Chromosome[float], ...],
        use_numpy: bool = False,
    ) -> FloatCodec[list[list[float]]]:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new FloatCodec instance with the specified chromosomes.
        """
        return FloatCodec(chromosomes=chromosomes, use_numpy=use_numpy)

    @staticmethod
    def matrix(
        shape: tuple[int, int] | list[int],
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ) -> FloatCodec[list[list[float]]]:
        """
        Create a matrix codec with specified rows and columns.
        Args:
            shape: A tuple (rows, cols) or a list of integers specifying the shape of the matrix.
            init_range: Minimum and maximum value for the Gene's Allele to be init with.
            bounds: Minimum and maximum values the allele is allowed to be within during evolution.
            use_numpy: Whether or not to decode the Genotype into a numpy array.
        Returns:
            A new FloatCodec instance with matrix configuration.

        Example
        --------
        Create a codec that will produce a Genotype with 2 Chromosomes both containing 3 FloatGenes
        Alleles between 0.0 and 1.0, and bounds between -1.0 and 2.0:
        >>> rd.FloatCodec.matrix(shape=(2, 3), init_range=(0.0, 1.0), bounds=(-1.0, 2.0))
        FloatCodec(...)

        The same can be achieved with a list of shapes:
        >>> rd.FloatCodec.matrix(shape=[3, 3], init_range=(0.0, 1.0), bounds=(-1.0, 2.0))
        FloatCodec(...)
        """
        return FloatCodec(
            shape=shape,
            init_range=init_range,
            bounds=bounds,
            use_numpy=use_numpy,
            dtype=dtype,
        )

    @staticmethod
    def vector(
        length: int,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        use_numpy: bool = False,
        dtype: DataTypeClass | DataType | None = None,
    ) -> FloatCodec[list[float]]:
        """
        Create a vector codec with specified length.
        Args:
            length: Length of the vector.
            init_range: Minimum and maximum value for the Gene's Allele to be init with.
            bounds: Minimum and maximum values the allele is allowed to be within during evolution.
            use_numpy: Whether or not to decode the Genotype into a numpy array.
        Returns:
            A new FloatCodec instance with vector configuration.

        Example
        --------
        Create a FloatCodec that will encode a Genotype with a single Chromosome containing 5 FloatGenes
        >>> rd.FloatCodec.vector(length=5, init_range=(0.0, 1.0), bounds=(-1.0, 2.0), use_numpy=False)
        FloatCodec(...)
        """
        return FloatCodec(
            shape=length,
            init_range=init_range,
            bounds=bounds,
            use_numpy=use_numpy,
            dtype=dtype,
        )

    @staticmethod
    def scalar(
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        dtype: DataTypeClass | DataType | None = None,
    ) -> FloatCodec[float]:
        """
        Create a scalar codec.
        Args:
            init_range: Minimum and maximum value for the Gene's Allele to be init with.
            bounds: Minimum and maximum values the allele is allowed to be within during evolution.
        Returns:
            A new FloatCodec instance with scalar configuration.

        Example:
        --------
        Create a FloatCodec that will encode a Genotype with a single Chromosome containing a single FloatGene
        with Alleles between 0.0 and 1.0, and bounds between -1.0 and 2.0:
        >>> rd.FloatCodec.scalar(init_range=(0.0, 1.0), bounds=(-1.0, 2.0))
        FloatCodec(...)
        """
        return FloatCodec(
            init_range=init_range,
            bounds=bounds,
            dtype=dtype,
        )

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
            if not issubclass(dtype, FloatType):
                raise ValueError("dtype must be a float data type.")
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
            if not issubclass(dtype, FloatType):
                raise ValueError("dtype must be a float data type.")
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
        shape: tuple[int, int] | list[int],
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
            if not issubclass(dtype, FloatType):
                raise ValueError("dtype must be a float data type.")
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
        genes: list[Gene[float]] | tuple[Gene[float], ...], use_numpy: bool = False
    ) -> PyFloatCodec:
        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")

        return PyFloatCodec.from_genes(
            list(map(lambda g: g.__backend__(), genes)), use_numpy=use_numpy
        )

    @staticmethod
    def __from_chromosomes(
        chromosomes: list[Chromosome[float]] | tuple[Chromosome[float], ...],
        use_numpy: bool = False,
    ) -> PyFloatCodec:
        from radiate.genome import GeneType

        if not isinstance(chromosomes, (list, tuple)):
            raise TypeError(
                "chromosomes must be a list or tuple of Chromosome instances."
            )
        if not all(g.gene_type() == GeneType.FLOAT for c in chromosomes for g in c):
            raise TypeError("All chromosomes must be of type 'float'.")

        return PyFloatCodec.from_chromosomes(
            list(map(lambda c: c.__backend__(), chromosomes)), use_numpy=use_numpy
        )
