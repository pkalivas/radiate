from __future__ import annotations

from radiate.radiate import PyPhenotype

from .._bridge import RsObject
from .._typing import AtLeastOne
from .gene import GeneType
from .genotype import Genotype


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
        :return: The gene type as a string.
        """
        from . import GeneType

        return GeneType.from_str(self.__backend__().genotype.gene_type())

    def id(self) -> int:
        """
        :return: The unique identifier of the phenotype.
        """
        return self.__backend__().id

    def generation(self) -> int:
        """
        :return: The generation of the phenotype.
        """
        return self.__backend__().generation

    def species(self) -> int:
        """
        :return: The species of the phenotype.
        """
        return self.__backend__().species

    def score(self) -> list[float]:
        """
        :return: The score of the phenotype.
        """
        return self.try_get_cache("score_cache", lambda: self.__backend__().score)

    def genotype(self) -> Genotype[T]:
        """
        :return: The genotype of the phenotype.
        """
        return self.try_get_cache(
            "genotype_cache", lambda: Genotype.from_rust(self.__backend__().genotype)
        )
