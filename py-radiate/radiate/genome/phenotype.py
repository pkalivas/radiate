from __future__ import annotations

from radiate.radiate import PyPhenotype
from radiate._typing import AtLeastOne
from radiate._bridge.wrapper import RsObject

from .genotype import Genotype
from .gene import GeneType


class Phenotype[T](RsObject):
    """
    Represents a phenotype in a genome.
    """

    def __init__(
        self,
        genotype: Genotype[T] | None = None,
        *,
        score: AtLeastOne[float] | None = None,
    ):
        super().__init__()

        if isinstance(genotype, Genotype):
            if isinstance(score, float):
                score = [score]

            self._pyobj = PyPhenotype(genotype=genotype.__backend__(), score=score)
        else:
            raise TypeError(f"Cannot create Phenotype with instance of {genotype}")

    def __repr__(self):
        return self.__backend__().__repr__()

    def __len__(self):
        """
        Returns the length of the phenotype.
        :return: Length of the phenotype.
        """
        return len(self.__backend__().genotype)

    def __hash__(self):
        res = hash(self.id())
        for score in self.score():
            res ^= hash(score)
        return res

    def gene_type(self) -> "GeneType":
        """
        Returns the type of the genes in the phenotype.
        :return: The gene type as a string.
        """
        from . import GeneType

        return GeneType.from_str(self.__backend__().genotype.gene_type())

    def id(self) -> int:
        """
        Returns the unique identifier of the phenotype.
        :return: The unique identifier of the phenotype.
        """
        return self.__backend__().id

    def score(self) -> list[float]:
        """
        Returns the score of the phenotype.
        :return: The score of the phenotype.
        """
        return self.try_get_cache("score_cache", lambda: self.__backend__().score)

    def genotype(self) -> Genotype[T]:
        """
        Returns the genotype of the phenotype.
        :return: The genotype of the phenotype.
        """
        return self.try_get_cache(
            "genotype_cache", lambda: Genotype.from_rust(self.__backend__().genotype)
        )
