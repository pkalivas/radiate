from __future__ import annotations
from typing import List

from radiate.genome.gene import GeneType
from radiate.genome.wrapper import PythonWrapper
from .genotype import Genotype
from radiate.radiate import PyPhenotype


class Phenotype[T](PythonWrapper[PyPhenotype]):
    """
    Represents a phenotype in a genome.
    """

    def __init__(
        self,
        genotype: Genotype[T] | None = None,
        *,
        score: List[float] | float | None = None,
    ):
        super().__init__()

        if isinstance(genotype, Genotype):
            if isinstance(score, float):
                score = [score]

            self._pyobj = PyPhenotype(genotype=genotype.to_python(), score=score)
        else:
            raise TypeError(f"Cannot create Phenotype with instance of {genotype}")

    def __repr__(self):
        return self._pyobj.__repr__()

    def __len__(self):
        """
        Returns the length of the phenotype.
        :return: Length of the phenotype.
        """
        return len(self._pyobj.genotype)

    def gene_type(self) -> GeneType:
        """
        Returns the type of the genes in the phenotype.
        :return: The gene type as a string.
        """
        return GeneType.from_str(self._pyobj.genotype.gene_type())

    def score(self) -> List[float]:
        """
        Returns the score of the phenotype.
        :return: The score of the phenotype.
        """
        return self._pyobj.score

    def genotype(self) -> Genotype[T]:
        """
        Returns the genotype of the phenotype.
        :return: The genotype of the phenotype.
        """
        return Genotype.from_python(self._pyobj.genotype)
