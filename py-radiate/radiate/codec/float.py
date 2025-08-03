from __future__ import annotations

from typing import List, Optional, Tuple, Union

from radiate.genome.gene import GeneType

from .base import CodecBase

from radiate.genome import Genotype, Gene, Chromosome

from radiate.radiate import PyFloatCodec

type CodecInput = Union[
    PyFloatCodec,
    Gene[float],
    List[Gene[float]],
    Chromosome[float],
    List[Chromosome[float]],
]


class FloatCodec[T](CodecBase[float, T]):
    def __init__(self, encoding: CodecInput):
        """
        Initialize the float codec with a PyFloatCodec instance.
        :param codec: An instance of PyFloatCodec.
        """
        if isinstance(encoding, Gene):
            encoding = [encoding]
        if isinstance(encoding, list) and all(isinstance(g, Gene) for g in encoding):
            encoding = PyFloatCodec.from_genes([g.to_python() for g in encoding])
        elif isinstance(encoding, list) and all(isinstance(c, Chromosome) for c in encoding):
            encoding = PyFloatCodec.from_chromosomes([c.to_python() for c in encoding])
        elif not isinstance(encoding, PyFloatCodec):
            raise TypeError("encoding must be a PyFloatCodec instance or a list of Gene/Chromosome instances.")

        self.codec = encoding

    def encode(self) -> Genotype[float]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_python(self.codec.encode_py())

    def decode(self, genotype: Genotype[float]) -> T:
        """
        Decode a Genotype into its float representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded float representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype=genotype.to_python())
    
    def _create_encoding(self, encoding: CodecInput) -> PyFloatCodec:
        """
        Create a PyFloatCodec from the provided encoding.
        :param encoding: The input encoding to create the codec from.
        :return: A PyFloatCodec instance.
        """
        if isinstance(encoding, PyFloatCodec):
            return encoding
        elif isinstance(encoding, Gene):
            return PyFloatCodec.from_genes([encoding])
        elif isinstance(encoding, Chromosome):
            return PyFloatCodec.from_chromosomes([encoding])
        elif isinstance(encoding, list):
            if all(isinstance(g, Gene) for g in encoding):
                return PyFloatCodec.from_genes([g for g in encoding])
            elif all(isinstance(c, Chromosome) for c in encoding):
                return PyFloatCodec.from_chromosomes([c for c in encoding])
            else:
                raise TypeError("Invalid list type for FloatCodec encoding.")
        else:
            raise TypeError("Invalid encoding type for FloatCodec.")

    @staticmethod
    def from_genes(
        genes: List[Gene[float]] | Tuple[Gene[float], ...], use_numpy: bool = False
    ) -> FloatCodec[List[float]]:
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
                list(map(lambda g: g.to_python(), genes)), use_numpy=use_numpy
            )
        )

    @staticmethod
    def from_chromosomes(
        chromosomes: List[Chromosome[float]] | Tuple[Chromosome[float], ...],
    ) -> FloatCodec[List[List[float]]]:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new FloatCodec instance with the specified chromosomes.
        """
        if not isinstance(chromosomes, (list, tuple)):
            raise TypeError(
                "chromosomes must be a list or tuple of Chromosome instances."
            )
        if not all(
            g.gene_type() == GeneType.FLOAT for c in chromosomes for g in c
        ):
            raise TypeError("All chromosomes must be of type 'float'.")

        return FloatCodec(
            PyFloatCodec.from_chromosomes(
                list(map(lambda c: c.to_python(), chromosomes))
            )
        )

    @staticmethod
    def matrix(
        shape: Tuple[int, int] | List[int],
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
        use_numpy: bool = False,
    ) -> FloatCodec[List[List[float]]]:
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
                use_numpy=use_numpy,
            )
        )

    @staticmethod
    def vector(
        length: int,
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
        use_numpy: bool = False,
    ) -> FloatCodec[List[float]]:
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
                use_numpy=use_numpy,
            )
        )

    @staticmethod
    def scalar(
        value_range: Optional[Tuple[float, float]] = None,
        bound_range: Optional[Tuple[float, float]] = None,
    ) -> FloatCodec[float]:
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
