from __future__ import annotations
from typing import List
from radiate.radiate import PyPhenotype
from .genotype import Genotype


class Phenotype:
    """
    Represents a phenotype in a genome.
    """

    def __init__(
        self,
        phenotype: PyPhenotype | None = None,
        genotype: Genotype | None = None,
        score: List[float] | float | None = None,
    ):
        """
        Initializes a Phenotype instance.

        :param genotype: A Genotype instance.
        """
        if isinstance(phenotype, PyPhenotype):
            self.__inner = phenotype
        elif isinstance(genotype, Genotype):
            if score is not None:
                score = score if isinstance(score, list) else [score]

            self.__inner = PyPhenotype(genotype.py_genotype(), score=score)
        else:
            raise TypeError("genotype must be an instance of Genotype or PyPhenotype")

    def __repr__(self):
        return f"Phenotype(genotype={self.__inner})"

    def __len__(self):
        """
        Returns the length of the phenotype.
        :return: Length of the phenotype.
        """
        return len(self.__inner.genotype.chromosomes)

    def __eq__(self, other: Phenotype) -> bool:
        """
        Checks if two Phenotype instances are equal.
        :param other: Another Phenotype instance.
        :return: True if both phenotypes are equal, False otherwise.
        """
        if not isinstance(other, Phenotype):
            return False
        return self.__inner == other.py_phenotype()

    def py_phenotype(self) -> PyPhenotype:
        """
        Returns the underlying PyPhenotype instance.
        :return: The PyPhenotype instance associated with this Phenotype.
        """
        return self.__inner

    def score(self) -> List[float]:
        """
        Returns the score of the phenotype.
        :return: The score of the phenotype.
        """
        return self.__inner.score

    def genotype(self) -> Genotype:
        """
        Returns the genotype of the phenotype.
        :return: The genotype of the phenotype.
        """
        return Genotype(genotype=self.__inner.genotype)

    def id(self) -> int:
        """
        Returns the ID of the phenotype.
        :return: The ID of the phenotype.
        """
        return self.__inner.id
