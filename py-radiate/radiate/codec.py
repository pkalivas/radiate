from typing import List, Optional, Tuple

from radiate.radiate import PyFloatCodec, PyIntCodec, PyCharCodec, PyBitCodec


class FloatCodec:
    def __init__(
        self,
        chromosomes: List[int],
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
    ):
        """
        Initialize the float codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
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

        self.codec = PyFloatCodec(
            chromosome_lengths=chromosomes,
            value_range=value_range,
            bound_range=bound_range,
        )


class IntCodec:
    def __init__(
        self,
        chromosomes: List[int],
        value_range: Optional[Tuple[int, int]] = None,
        bound_range: Optional[Tuple[int, int]] = None,
    ):
        """
        Initialize the int codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
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

        self.codec = PyIntCodec(
            chromosome_lengths=chromosomes,
            value_range=value_range,
            bound_range=bound_range,
        )


class CharCodec:
    def __init__(self, chromosomes: List[int], char_set: str | List[str] = None):
        """
        Initialize the char codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        :param value_range: Minimum and maximum value for the genes.
        """

        if isinstance(char_set, str):
            char_set = list(char_set)

        if char_set is not None:
            for char in char_set:
                if not isinstance(char, str) or len(char) != 1:
                    raise ValueError(
                        "Character set must be a string or list of single-character strings."
                    )

        self.codec = PyCharCodec(
            chromosome_lengths=chromosomes,
            char_set=''.join(set(char_set)) if char_set else None,
        )


class BitCodec:
    """
    BitCodec is a class that represents a codec for bit-based chromosomes.
    It is used to encode and decode chromosomes into bit strings.
    """

    def __init__(self, chromosomes: List[int]):
        """
        Initialize the bit codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        """
        self.codec = PyBitCodec(
            chromosome_lengths=chromosomes,
        )
