from typing import List, Optional, Tuple
from .codec import CodecBase

from radiate.radiate import PyFloatCodec


class FloatCodec(CodecBase):
    def __init__(self, codec: PyFloatCodec):
        """
        Initialize the float codec with a PyFloatCodec instance.
        :param codec: An instance of PyFloatCodec.
        """
        if not isinstance(codec, PyFloatCodec):
            raise TypeError("codec must be an instance of PyFloatCodec.")
        self.codec = codec

    @staticmethod
    def matrix(
        shape: Tuple[int, int] | List[int],
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
    ) -> "FloatCodec":
        """
        Create a matrix codec with specified rows and columns.
        Args:
            shape: A tuple (rows, cols) or a list of integers specifying the shape of the matrix.
            value_range: Minimum and maximum value for the Gene's Allele to be init with.
            bound_range: Minimum and maximum values the allele is allowed to be within during evolution
        Returns:
            A new FloatCodec instance with matrix configuration.

        Example
        --------
        Create a codec that will produce a Genotype with 2 Chromosomes both containing 3 FloatGenes
        Alleles between 0.0 and 1.0, and bounds between -1.0 and 2.0:
        >>> rd.FloatCodec.matrix(shape=(2, 3), value_range=(0.0, 1.0), bound_range=(-1.0, 2.0))
        FloatCodec(...)

        The same can be achieved with a list of shapes:
        >>> rd.FloatCodec.matrix(shape=[3, 3], value_range=(0.0, 1.0), bound_range=(-1.0, 2.0))
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

        if value_range is not None:
            if len(value_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if value_range[0] >= value_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
        if bound_range is not None:
            if len(bound_range) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bound_range[0] >= bound_range[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")

        return FloatCodec(
            PyFloatCodec.matrix(
                chromosome_lengths=shapes,
                value_range=value_range,
                bound_range=bound_range,
            )
        )

    @staticmethod
    def vector(
        length: int,
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
    ) -> "FloatCodec":
        """
        Create a vector codec with specified length.
        Args:
            length: Length of the vector.
            value_range: Minimum and maximum value for the Gene's Allele to be init with.
            bound_range: Minimum and maximum values the allele is allowed to be within during evolution.
        Returns:
            A new FloatCodec instance with vector configuration.

        Example
        --------
        Create a FloatCodec that will encode a Genotype with a single Chromosome containing 5 FloatGenes
        >>> rd.FloatCodec.vector(length=5, value_range=(0.0, 1.0), bound_range=(-1.0, 2.0))
        FloatCodec(...)
        """
        if length <= 0:
            raise ValueError("Length must be a positive integer.")

        if value_range is not None:
            if len(value_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if value_range[0] >= value_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
        if bound_range is not None:
            if len(bound_range) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bound_range[0] >= bound_range[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")

        return FloatCodec(
            PyFloatCodec.vector(
                length=length,
                value_range=value_range,
                bound_range=bound_range,
            )
        )

    @staticmethod
    def scalar(
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
    ) -> "FloatCodec":
        """
        Create a scalar codec.
        Args:
            value_range: Minimum and maximum value for the Gene's Allele to be init with.
            bound_range: Minimum and maximum values the allele is allowed to be within during evolution.
        Returns:
            A new FloatCodec instance with scalar configuration.

        Example:
        --------
        Create a FloatCodec that will encode a Genotype with a single Chromosome containing a single FloatGene
        with Alleles between 0.0 and 1.0, and bounds between -1.0 and 2.0:
        >>> rd.FloatCodec.scalar(value_range=(0.0, 1.0), bound_range=(-1.0, 2.0))
        FloatCodec(...)
        """
        if value_range is not None:
            if len(value_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if value_range[0] >= value_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
        if bound_range is not None:
            if len(bound_range) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bound_range[0] >= bound_range[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")

        return FloatCodec(
            PyFloatCodec.scalar(
                value_range=value_range,
                bound_range=bound_range,
            )
        )
