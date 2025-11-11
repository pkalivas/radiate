from __future__ import annotations

from .base import CodecBase

from radiate._typing import FloatEncoding
from radiate.genome import Genotype, Gene, Chromosome

from radiate.radiate import PyFloatCodec


class FloatCodec[T](CodecBase[float, T]):
    def __init__(
        self, encoding: FloatEncoding | PyFloatCodec, *, use_numpy: bool = False
    ):
        """
        Initialize the float codec with a PyFloatCodec instance.
        :param codec: An instance of PyFloatCodec.
        """
        self.codec = self.__create_encoding(encoding, use_numpy)

    def encode(self) -> Genotype[float]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.codec.encode_py())

    def decode(self, genotype: Genotype[float]) -> T:
        """
        Decode a Genotype into its float representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded float representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype=genotype.__backend__())

    def __create_encoding(
        self, encoding: FloatEncoding, use_numpy: bool
    ) -> PyFloatCodec:
        """
        Create a PyFloatCodec from the provided encoding.
        :param encoding: The input encoding to create the codec from.
        :return: A PyFloatCodec instance.
        """
        if isinstance(encoding, PyFloatCodec):
            return encoding
        elif isinstance(encoding, Gene):
            return PyFloatCodec.from_genes(
                [encoding.__backend__()], use_numpy=use_numpy
            )
        elif isinstance(encoding, Chromosome):
            return PyFloatCodec.from_chromosomes(
                [encoding.__backend__()], use_numpy=use_numpy
            )
        elif isinstance(encoding, list):
            if all(isinstance(g, Gene) for g in encoding):
                return PyFloatCodec.from_genes(
                    [g.__backend__() for g in encoding], use_numpy=use_numpy
                )
            elif all(isinstance(c, Chromosome) for c in encoding):
                return PyFloatCodec.from_chromosomes(
                    [c.__backend__() for c in encoding], use_numpy=use_numpy
                )
            else:
                raise TypeError("Invalid list type for FloatCodec encoding.")
        else:
            raise TypeError(f"Invalid encoding type for FloatCodec - {type(encoding)}.")

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
        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")

        return FloatCodec(
            PyFloatCodec.from_genes(
                list(map(lambda g: g.__backend__(), genes)), use_numpy=use_numpy
            )
        )

    @staticmethod
    def from_chromosomes(
        chromosomes: list[Chromosome[float]] | tuple[Chromosome[float], ...],
    ) -> FloatCodec[list[list[float]]]:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new FloatCodec instance with the specified chromosomes.
        """
        from radiate.genome import GeneType

        if not isinstance(chromosomes, (list, tuple)):
            raise TypeError(
                "chromosomes must be a list or tuple of Chromosome instances."
            )
        if not all(g.gene_type() == GeneType.FLOAT for c in chromosomes for g in c):
            raise TypeError("All chromosomes must be of type 'float'.")

        return FloatCodec(
            PyFloatCodec.from_chromosomes(
                list(map(lambda c: c.__backend__(), chromosomes))
            )
        )

    @staticmethod
    def matrix(
        shape: tuple[int, int] | list[int],
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        use_numpy: bool = False,
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

        return FloatCodec(
            PyFloatCodec.matrix(
                chromosome_lengths=shapes,
                value_range=init_range,
                bound_range=bounds,
                use_numpy=use_numpy,
            )
        )

    @staticmethod
    def vector(
        length: int,
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
        use_numpy: bool = False,
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

        return FloatCodec(
            PyFloatCodec.vector(
                length=length,
                value_range=init_range,
                bound_range=bounds,
                use_numpy=use_numpy,
            )
        )

    @staticmethod
    def scalar(
        init_range: tuple[float, float] | None = None,
        bounds: tuple[float, float] | None = None,
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

        return FloatCodec(
            PyFloatCodec.scalar(
                value_range=init_range,
                bound_range=bounds,
            )
        )
