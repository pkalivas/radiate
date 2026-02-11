from __future__ import annotations


from .base import CodecBase

from radiate.radiate import PyBitCodec
from radiate.genome import Genotype, Gene, Chromosome, GeneType
from radiate._bridge.wrapper import RsObject


class BitCodec[T](CodecBase[bool, T], RsObject[PyBitCodec]):
    """BitCodec for bit-based chromosomes. Encodes/decodes to bit strings."""

    gene_type = GeneType.BIT

    def __init__(
        self,
        shape: int | tuple[int, int] | list[int] | None = None,
        genes: Gene[bool] | list[Gene[bool]] | tuple[Gene[bool], ...] | None = None,
        chromosomes: Chromosome[bool]
        | list[Chromosome[bool]]
        | tuple[Chromosome[bool], ...]
        | None = None,
        use_numpy: bool = False,
    ):
        """
        Initialize the bit codec with number of chromosomes and value bounds.
        :param chromosomes: Number of chromosomes with the number of genes in each chromosome.
        """
        if shape is not None:
            if isinstance(shape, int):
                self._pyobj = self.__vector(length=shape, use_numpy=use_numpy)
            elif isinstance(shape, (tuple, list)):
                self._pyobj = self.__matrix(shape=shape, use_numpy=use_numpy)
            else:
                raise ValueError("Shape must be an int, tuple of ints, or list of ints.")
        elif genes is not None:
            self._pyobj = self.__from_genes(genes=genes, use_numpy=use_numpy)
        elif chromosomes is not None:
            self._pyobj = self.__from_chromosomes(
                chromosomes=chromosomes, use_numpy=use_numpy
            )
        else:
            raise ValueError("Shape must be provided.")

    def encode(self) -> Genotype[bool]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype[bool]) -> T:
        """
        Decode a Genotype into its bit representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded bit representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.__backend__().decode_py(genotype.__backend__())
    
    @staticmethod
    def from_genes(
        genes: list[Gene[bool]] | tuple[Gene[bool], ...], use_numpy: bool = False
    ) -> BitCodec[list[bool]]:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new BitCodec instance with the specified genes.
        """
        return BitCodec(
            genes=genes,
            use_numpy=use_numpy,
        )

    @staticmethod
    def from_chromosomes(
        chromosomes: list[Chromosome[bool]] | tuple[Chromosome[bool], ...],
        use_numpy: bool = False,
    ) -> BitCodec[list[list[bool]]]:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new BitCodec instance with the specified chromosomes.
        """
        return BitCodec(chromosomes=chromosomes, use_numpy=use_numpy)

    @staticmethod
    def matrix(
        shape: list[int] | tuple[int, int], use_numpy: bool = False
    ) -> BitCodec[list[list[bool]]]:
        """
        Initialize the bit codec with a matrix of chromosomes.
        Args:
            shape: A list of integers specifying the shape of the matrix.
        Returns:
            A new BitCodec instance with matrix configuration.

        Example
        --------
        >>> rd.BitCodec.matrix(chromosome_lengths=[5, 5])
        BitCodec(...)
        """
        return BitCodec(shape=shape, use_numpy=use_numpy)

    @staticmethod
    def vector(length: int = 8, use_numpy: bool = False) -> BitCodec[list[bool]]:
        """
        Initialize the bit codec with a single chromosome of specified length.
        Args:
            length: Length of the chromosome.
        Returns:
            A new BitCodec instance with vector configuration.

        Example
        --------
        >>> rd.BitCodec.vector(length=5)
        BitCodec(...)
        """
        return BitCodec(shape=length, use_numpy=use_numpy)

    @staticmethod
    def __matrix(
        shape: list[int] | tuple[int, int], use_numpy: bool = False
    ) -> BitCodec[list[list[bool]]]:
        """
        Initialize the bit codec with a matrix of chromosomes.
        Args:
            shape: A list of integers specifying the shape of the matrix.
        Returns:
            A new BitCodec instance with matrix configuration.

        Example
        --------
        >>> rd.BitCodec.matrix(chromosome_lengths=[5, 5])
        BitCodec(...)
        """
        if isinstance(shape, tuple):
            if len(shape) != 2:
                raise ValueError("Shape must be a tuple of (rows, cols).")
            rows, cols = shape
            if rows < 1 or cols < 1:
                raise ValueError("Rows and columns must be at least 1.")
            shape = [cols for _ in range(rows)]
        elif isinstance(shape, list):
            if not all(isinstance(x, int) and x > 0 for x in shape):
                raise ValueError("Shape must be a list of positive integers.")

        return PyBitCodec.matrix(chromosome_lengths=shape, use_numpy=use_numpy)

    @staticmethod
    def __vector(length: int = 8, use_numpy: bool = False) -> BitCodec[list[bool]]:
        """
        Initialize the bit codec with a single chromosome of specified length.
        Args:
            length: Length of the chromosome.
        Returns:
            A new BitCodec instance with vector configuration.

        Example
        --------
        >>> rd.BitCodec.vector(length=5)
        BitCodec(...)
        """
        return PyBitCodec.vector(chromosome_length=length, use_numpy=use_numpy)

    @staticmethod
    def __from_genes(
        genes: list[Gene[bool]] | tuple[Gene[bool], ...], use_numpy: bool = False
    ) -> PyBitCodec:
        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")

        return PyBitCodec.from_genes(
            list(map(lambda g: g.__backend__(), genes)), use_numpy=use_numpy
        )

    @staticmethod
    def __from_chromosomes(
        chromosomes: list[Chromosome[bool]] | tuple[Chromosome[bool], ...],
        use_numpy: bool = False,
    ) -> PyBitCodec:
        from radiate.genome import GeneType

        if not isinstance(chromosomes, (list, tuple)):
            raise TypeError(
                "chromosomes must be a list or tuple of Chromosome instances."
            )
        if not all(g.gene_type() == GeneType.BOOL for c in chromosomes for g in c):
            raise TypeError("All chromosomes must be of type 'bool'.")

        return PyBitCodec.from_chromosomes(
            list(map(lambda c: c.__backend__(), chromosomes)), use_numpy=use_numpy
        )