from typing import List, Optional, Tuple
from .codec import CodecBase
from radiate.radiate import PyIntCodec


class IntCodec(CodecBase):
    def __init__(self, codec: PyIntCodec):
        """
        Initialize the int codec with a PyIntCodec instance.
        :param codec: An instance of PyIntCodec.
        """
        if not isinstance(codec, PyIntCodec):
            raise TypeError("codec must be an instance of PyIntCodec.")
        self.codec = codec

    @staticmethod
    def matrix(
        shape: Tuple[int, int] | List[int],
        value_range: Optional[Tuple[int, int]] = None,
        bound_range: Optional[Tuple[int, int]] = None,
    ) -> "IntCodec":
        """
        Initialize the int codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
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

        return IntCodec(
            PyIntCodec.matrix(
                chromosome_lengths=shapes,
                value_range=value_range,
                bound_range=bound_range,
            )
        )

    @staticmethod
    def vector(
        length: int,
        value_range: Optional[Tuple[int, int]] = None,
        bound_range: Optional[Tuple[int, int]] = None,
    ) -> "IntCodec":
        """
        Create a vector codec with specified length.
        :param length: Length of the vector.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
        :return: A new IntCodec instance with vector configuration.
        """
        if length <= 0:
            raise ValueError("Length must be greater than 0.")
        return IntCodec(
            PyIntCodec.vector(
                length=length,
                value_range=value_range,
                bound_range=bound_range,
            )
        )

    @staticmethod
    def scalar(
        value_range: Optional[Tuple[int, int]] = None,
        bound_range: Optional[Tuple[int, int]] = None,
    ) -> "IntCodec":
        """
        Create a scalar codec with specified value and bound ranges.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new IntCodec instance with scalar configuration.
        """
        return IntCodec(
            PyIntCodec.scalar(
                value_range=value_range,
                bound_range=bound_range,
            )
        )
