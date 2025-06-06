from __future__ import annotations

from radiate.radiate import PyPhenotype
from .genotype import Genotype


class Phenotype:
    """
    Represents a phenotype in a genome.
    """

    def __init__(
        self, phenotype: PyPhenotype | None = None, genotype: Genotype | None = None
    ):
        """
        Initializes a Phenotype instance.

        :param genotype: A Genotype instance.
        """
        if isinstance(phenotype, PyPhenotype):
            self.__inner = phenotype
        elif isinstance(genotype, Genotype):
            self.__inner = PyPhenotype(genotype.py_genotype())
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

    def py_phenotype(self) -> PyPhenotype:
        """
        Returns the underlying PyPhenotype instance.
        :return: The PyPhenotype instance associated with this Phenotype.
        """
        return self.__inner

    def genotype(self) -> Genotype:
        """
        Returns the genotype of the phenotype.
        :return: The genotype of the phenotype.
        """
        return Genotype(genotype=self.__inner.genotype)
