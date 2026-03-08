from __future__ import annotations

from typing import overload, Sequence, Any

from radiate.genome.chromosome import Chromosome
from radiate.genome.gene import Gene
from .base import CodecBase

from radiate.radiate import PyCharCodec
from radiate.genome import Genotype, GeneType
from radiate._bridge.wrapper import RsObject
from radiate._typing import AtLeastOne, MatrixDecoding


def _normalize_char_set(char_set: str | list[str] | set[str] | None) -> str | None:
    if isinstance(char_set, str):
        return "".join(set(char_set))
    elif isinstance(char_set, (list, set)):
        return "".join(set(char_set))
    elif char_set is None:
        return None
    else:
        raise ValueError(
            "char_set must be a string, list of strings, set of strings, or None."
        )


class CharCodec[D](CodecBase[str, D], RsObject):
    gene_type = GeneType.CHAR

    @overload
    def __new__(
        cls,
        shape: int,
        *,
        char_set: str | list[str] | set[str] | None = None,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "CharCodec[list[str]]": ...

    @overload
    def __new__(
        cls,
        shape: Sequence[int],
        *,
        char_set: str | list[str] | set[str] | None = None,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "CharCodec[MatrixDecoding[str]]": ...

    def __new__(cls, *args: Any, **kwargs: Any) -> "CharCodec[Any]":
        return super().__new__(cls)

    def __init__(
        self,
        shape: AtLeastOne[int] | None = None,
        char_set: str | list[str] | set[str] | None = None,
        genes: Gene[str] | Sequence[Gene[str]] | None = None,
        chromosomes: Chromosome[str] | Sequence[Chromosome[str]] | None = None,
    ):
        """
        Initialize the char codec with number of chromosomes and value bounds.
        :param shape: Number of chromosomes with the number of genes in each chromosome.
        :param char_set: Set of characters to use for the genes.
        """
        if shape is not None:
            if isinstance(shape, int):
                self._pyobj = PyCharCodec.vector(
                    length=shape, char_set=_normalize_char_set(char_set)
                )
            elif isinstance(shape, (tuple, list)):
                self._pyobj = self.__matrix(
                    shape=shape,
                    char_set=set(char_set) if char_set is not None else None,
                )
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

    def decode(self, genotype: Genotype[str]) -> D:
        """
        Decode a Genotype into its character representation.
        :param genotype: A Genotype instance to decode.
        :return: The decoded character representation of the Genotype.
        """
        if not isinstance(genotype, Genotype):
            raise TypeError("genotype must be an instance of Genotype.")
        return self.__backend__().decode_py(genotype=genotype.__backend__())

    # @staticmethod
    # def from_genes(genes: Gene[str] | Sequence[Gene[str]]) -> CharCodec:
    #     """
    #     Create a codec for a single chromosome with specified genes.
    #     Args:
    #         genes: A list or tuple of Gene instances.
    #     Returns:
    #         A new CharCodec instance with the specified genes.
    #     """
    #     return CharCodec(genes=genes)

    # @staticmethod
    # def from_chromosomes(
    #     chromosomes: Chromosome[str] | Sequence[Chromosome[str]],
    # ) -> CharCodec:
    #     """
    #     Create a codec for multiple chromosomes.
    #     Args:
    #         chromosomes: A list or tuple of Chromosome instances.
    #     Returns:
    #         A new CharCodec instance with the specified chromosomes.
    #     """
    #     return CharCodec(chromosomes=chromosomes)

    @staticmethod
    def matrix(
        shape: list[int] | tuple[int, int],
        char_set: str | list[str] | set[str] | None = None,
    ) -> CharCodec[MatrixDecoding[str]]:
        """
        Initialize the char codec with number of chromosomes and value bounds.
        Args:
            shape: A list of integers specifying the lengths of each chromosome.
            char_set: A set of characters to use for the genes.
        Returns:
            A new CharCodec instance with matrix configuration.

        Example
        --------
        >>> rd.CharCodec.matrix(shape=[5, 5], char_set={"0", "1"})
        CharCodec(...)
        """
        return CharCodec(shape=shape, char_set=char_set)

    @staticmethod
    def vector(
        length: int, char_set: str | list[str] | set[str] | None = None
    ) -> CharCodec[list[str]]:
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
    def __from_genes(genes: AtLeastOne[Gene[str]]) -> CharCodec[list[str]]:
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
        chromosomes: AtLeastOne[Chromosome[str]],
    ) -> CharCodec[MatrixDecoding[str]]:
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
            list(map(lambda c: c.__backend__(), chromosomes))
        )

    @staticmethod
    def __vector(
        length: AtLeastOne[int], char_set: str | list[str] | set[str] | None = None
    ) -> PyCharCodec:
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
        return PyCharCodec.vector(length, _normalize_char_set(char_set))

    @staticmethod
    def __matrix(
        shape: AtLeastOne[int],
        char_set: set[str] | None = None,
    ) -> PyCharCodec:
        """
        Initialize the char codec with number of chromosomes and value bounds.
        Args:
            shape: A list of integers specifying the lengths of each chromosome.
            char_set: A string or list of strings representing the character set.
        Returns:
            A new PyCharCodec instance with matrix configuration.

        Example
        --------
        >>> rd.CharCodec.matrix(shape=[5, 5], char_set="01")
        CharCodec(...)
        """

        if isinstance(shape, tuple):
            if len(shape) != 2:
                raise ValueError("Shape must be a tuple of (rows, cols).")
            rows, cols = shape
            if rows < 1 or cols < 1:
                raise ValueError("Rows and columns must be at least 1.")
            shape = [cols for _ in range(rows)]
        if isinstance(shape, list):
            if not all(isinstance(x, int) and x > 0 for x in shape):
                raise ValueError("Shape must be a list of positive integers.")

        if char_set is not None:
            for char in char_set:
                if not isinstance(char, str) or len(char) != 1:
                    raise ValueError(
                        "Character set must be a string or list of single-character strings."
                    )

        return PyCharCodec.matrix(shape, _normalize_char_set(char_set))
