from __future__ import annotations

from radiate.genome.chromosome import Chromosome
from radiate.genome.gene import Gene
from .base import CodecBase

from radiate.radiate import PyCharCodec
from radiate.genome import Genotype
from radiate.wrapper import PyObject


class CharCodec[T](CodecBase[str, T], PyObject[PyCharCodec]):
    def __init__(
        self,
        shape: int | tuple[int, int] | list[int] | None = None,
        char_set: str | list[str] = None,
        genes: Gene[str] | list[Gene[str]] | tuple[Gene[str], ...] | None = None,
        chromosomes: Chromosome[str]
        | list[Chromosome[str]]
        | tuple[Chromosome[str], ...]
        | None = None,
    ):
        """
        Initialize the char codec with number of chromosomes and value bounds.
        :param shape: Number of chromosomes with the number of genes in each chromosome.
        :param init_range: Range for initializing gene values.
        :param bounds: Bounds for gene values.
        """
        if shape is not None:
            if isinstance(shape, int):
                self._pyobj = self.__vector(length=shape, char_set=char_set)
            elif isinstance(shape, (tuple, list)):
                self._pyobj = self.__matrix(chromosomes=shape, char_set=char_set)
            else:
                raise ValueError(
                    "Shape must be an int, tuple of ints, or list of ints."
                )
        elif genes is not None:
            self._pyobj = self.__from_genes(genes=genes)
        elif chromosomes is not None:
            self._pyobj = self.__from_chromosomes(chromosomes=chromosomes)
        else:
            raise ValueError("Shape must be provided.")

    def encode(self) -> Genotype[str]:
        """
        Encode the codec into a Genotype.
        :return: A Genotype instance.
        """
        return Genotype.from_rust(self.__backend__().encode_py())

    def decode(self, genotype: Genotype[str]) -> T:
        """
        Decode a Genotype into its character representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded character representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.__backend__().decode_py(genotype=genotype.__backend__())

    @staticmethod
    def from_genes(genes: list[Gene[str]] | tuple[Gene[str], ...]) -> CharCodec[str]:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new CharCodec instance with the specified genes.
        """
        return CharCodec(genes=genes)

    @staticmethod
    def from_chromosomes(
        chromosomes: list[Chromosome[str]] | tuple[Chromosome[str], ...],
    ) -> CharCodec[list[str]] | CharCodec[str]:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new CharCodec instance with the specified chromosomes.
        """
        return CharCodec(chromosomes=chromosomes)

    @staticmethod
    def matrix(
        chromosomes: list[int] | tuple[int, int],
        char_set: str | list[str] = None,
    ) -> CharCodec[list[list[str]]]:
        """
        Initialize the char codec with number of chromosomes and value bounds.
        Args:
            chromosomes: A list of integers specifying the lengths of each chromosome.
            char_set: A string or list of strings representing the character set.
        Returns:
            A new CharCodec instance with matrix configuration.

        Example
        --------
        >>> rd.CharCodec.matrix(chromosomes=[5, 5], char_set="01")
        CharCodec(...)
        """
        return CharCodec(shape=chromosomes, char_set=char_set)

    @staticmethod
    def vector(length: int, char_set: str | list[str] = None) -> CharCodec[list[str]]:
        """
        Initialize the char codec with a single chromosome of specified length.
        Args:
            length: Length of the chromosome.
            char_set: Character set to use for encoding.
        Returns:
            A new CharCodec instance with vector configuration.

        Example
        --------
        >>> rd.CharCodec.vector(length=5, char_set="01")
        CharCodec(...)
        """
        return CharCodec(shape=length, char_set=char_set)

    @staticmethod
    def __from_genes(genes: list[Gene[str]] | tuple[Gene[str], ...]) -> CharCodec[str]:
        """
        Create a codec for a single chromosome with specified genes.
        Args:
            genes: A list or tuple of Gene instances.
        Returns:
            A new CharCodec instance with the specified genes.
        """
        from radiate.genome import GeneType

        if not isinstance(genes, (list, tuple)):
            raise TypeError("genes must be a list or tuple of Gene instances.")
        if not all(g.gene_type() == GeneType.CHAR for g in genes):
            raise TypeError("All genes must be of type 'char'.")

        return PyCharCodec.from_genes(list(map(lambda g: g.__backend__(), genes)))

    @staticmethod
    def __from_chromosomes(
        chromosomes: list[Chromosome[str]] | tuple[Chromosome[str], ...],
    ) -> PyCharCodec:
        """
        Create a codec for multiple chromosomes.
        Args:
            chromosomes: A list or tuple of Chromosome instances.
        Returns:
            A new PyCharCodec instance with the specified chromosomes.
        """
        from radiate.genome import GeneType

        if not isinstance(chromosomes, (list, tuple)):
            raise TypeError(
                "chromosomes must be a list or tuple of Chromosome instances."
            )
        if not all(
            g.gene_type() == GeneType.CHAR for c in chromosomes for g in c.genes()
        ):
            raise TypeError("All chromosomes must be of type 'char'.")

        return PyCharCodec.from_chromosomes(
            list(map(lambda c: c.py_chromosome(), chromosomes))
        )

    @staticmethod
    def __vector(length: int, char_set: str | list[str] = None) -> PyCharCodec:
        """
        Initialize the char codec with a single chromosome of specified length.
        Args:
            length: Length of the chromosome.
            char_set: Character set to use for encoding.
        Returns:
            A new PyCharCodec instance with vector configuration.

        Example
        --------
        >>> rd.CharCodec.vector(length=5, char_set="01")
        CharCodec(...)
        """
        return PyCharCodec.vector(length, char_set)

    @staticmethod
    def __matrix(
        chromosomes: list[int] | tuple[int, int],
        char_set: str | list[str] = None,
    ) -> PyCharCodec:
        """
        Initialize the char codec with number of chromosomes and value bounds.
        Args:
            chromosomes: A list of integers specifying the lengths of each chromosome.
            char_set: A string or list of strings representing the character set.
        Returns:
            A new PyCharCodec instance with matrix configuration.

        Example
        --------
        >>> rd.CharCodec.matrix(chromosomes=[5, 5], char_set="01")
        CharCodec(...)
        """

        if isinstance(chromosomes, tuple):
            if len(chromosomes) != 2:
                raise ValueError("Chromosomes must be a tuple of (rows, cols).")
            rows, cols = chromosomes
            if rows < 1 or cols < 1:
                raise ValueError("Rows and columns must be at least 1.")
            chromosomes = [cols for _ in range(rows)]
        if isinstance(chromosomes, list):
            if not all(isinstance(x, int) and x > 0 for x in chromosomes):
                raise ValueError("Chromosomes must be a list of positive integers.")

        if char_set is not None:
            for char in char_set:
                if not isinstance(char, str) or len(char) != 1:
                    raise ValueError(
                        "Character set must be a string or list of single-character strings."
                    )

        return PyCharCodec.matrix(chromosomes, char_set)
