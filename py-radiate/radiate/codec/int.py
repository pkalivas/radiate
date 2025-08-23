from __future__ import annotations

from radiate.genome.chromosome import Chromosome
from radiate.genome.gene import Gene, GeneType

from .base import CodecBase
from radiate.genome import Genotype
from radiate.radiate import PyIntCodec


class IntCodec[D](CodecBase[int, D]):
    def __init__(self, codec: PyIntCodec):
        """
        Initialize the int codec with a PyIntCodec instance.
        :param codec: An instance of PyIntCodec.
        """
        if not isinstance(codec, PyIntCodec):
            raise TypeError("codec must be an instance of PyIntCodec.")
        self.codec = codec

    def encode(self) -> Genotype[int]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_python(self.codec.encode_py())

    def decode(self, genotype: Genotype[int]) -> D:
        """
        Decode a Genotype into its integer representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded integer representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.codec.decode_py(genotype=genotype.to_python())

    @staticmethod
    def from_genes(
        genes: list[Gene[int]] | tuple[Gene[int], ...], use_numpy: bool = False
    ) -> IntCodec[list[int]]:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new FloatCodec instance with the specified genes.
        """
        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")
        if not all(g.gene_type() == GeneType.INT for g in genes):
            raise TypeError("All genes must be of type 'int'.")

        return IntCodec(
            PyIntCodec.from_genes(
                list(map(lambda g: g.py_gene(), genes)), use_numpy=use_numpy
            )
        )

    @staticmethod
    def from_chromosomes(
        chromosomes: list[Chromosome[int]] | tuple[Chromosome[int], ...],
    ) -> IntCodec[list[list[int]]]:
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
            g.gene_type() == GeneType.INT for c in chromosomes for g in c.genes()
        ):
            raise TypeError("All chromosomes must be of type 'int'.")

        return IntCodec(
            PyIntCodec.from_chromosomes(
                list(map(lambda c: c.py_chromosome(), chromosomes))
            )
        )

    @staticmethod
    def matrix(
        shape: tuple[int, int] | list[int],
        value_range: tuple[int, int] | None = None,
        bound_range: tuple[int, int] | None = None,
        use_numpy: bool = False,
    ) -> IntCodec[list[list[int]]]:
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
                use_numpy=use_numpy,
            )
        )

    @staticmethod
    def vector(
        length: int,
        value_range: tuple[int, int] | None = None,
        bound_range: tuple[int, int] | None = None,
        use_numpy: bool = False,
    ) -> IntCodec[list[int]]:
        """
        Create a vector codec with specified length.
        :param length: Length of the vector.
        :param value_range: Minimum and maximum value for the genes.
        :param bound_range: Minimum and maximum bound for the genes.
        :return: A new IntCodec instance with vector configuration.
        """
        if length <= 0:
            raise ValueError("Length must be a positive integer.")
        if value_range is not None:
            if len(value_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if value_range[0] >= value_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
            if value_range[1] < value_range[0]:
                raise ValueError("Maximum value must be non-negative.")
        if bound_range is not None:
            if len(bound_range) != 2:
                raise ValueError("Bound range must be a tuple of (min, max).")
            if bound_range[0] >= bound_range[1]:
                raise ValueError("Minimum bound must be less than maximum bound.")
            if bound_range[1] < value_range[0]:
                raise ValueError("Maximum bound must be non-negative.")
        return IntCodec(
            PyIntCodec.vector(
                length=length,
                value_range=value_range,
                bound_range=bound_range,
                use_numpy=use_numpy,
            )
        )

    @staticmethod
    def scalar(
        value_range: tuple[int, int] | None = None,
        bound_range: tuple[int, int] | None = None,
    ) -> IntCodec[int]:
        """
        Create a scalar codec with specified value and bound ranges.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new IntCodec instance with scalar configuration.
        """
        if value_range is not None:
            if len(value_range) != 2:
                raise ValueError("Value range must be a tuple of (min, max).")
            if value_range[0] >= value_range[1]:
                raise ValueError("Minimum value must be less than maximum value.")
            if value_range[1] < value_range[0]:
                raise ValueError("Maximum value must be non-negative.")

        return IntCodec(
            PyIntCodec.scalar(
                value_range=value_range,
                bound_range=bound_range,
            )
        )
