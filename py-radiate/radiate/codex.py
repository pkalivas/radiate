from typing import List, Optional, Tuple

from radiate.radiate import (
    PyFloatCodex,
    PyIntCodex
)

class FloatCodex:

    def __init__(self,
                 chromosomes: List[int],
                 value_range: Optional[Tuple[float, float]] = None,
                 bound_range: Optional[Tuple[float, float]] = None):
        """
        Initialize the float codex with number of chromosomes and value bounds.
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
            
        self.codex = PyFloatCodex(
            chromosome_lengths=chromosomes,
            value_range=value_range,
            bound_range=bound_range,
        )


class IntCodex:

    def __init__(self,
                 chromosomes: List[int],
                 value_range: Optional[Tuple[int, int]] = None,
                 bound_range: Optional[Tuple[int, int]] = None):
        """
        Initialize the int codex with number of chromosomes and value bounds.
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
            
        self.codex = PyIntCodex(
            chromosome_lengths=chromosomes,
            value_range=value_range,
            bound_range=bound_range,
        )