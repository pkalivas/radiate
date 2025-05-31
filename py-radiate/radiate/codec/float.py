from typing import List, Optional, Tuple

from radiate.radiate import PyFloatCodec


class FloatCodec:
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
        :param rows: Number of rows in the matrix.
        :param cols: Number of columns in the matrix.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
        :return: A new FloatCodec instance with matrix configuration.
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
        :param length: Length of the vector.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
        :return: A new FloatCodec instance with vector configuration.
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
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
        :return: A new FloatCodec instance with scalar configuration.
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
