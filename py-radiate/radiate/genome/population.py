from __future__ import annotations

from typing import List
from radiate.genome.genotype import Genotype
from radiate.radiate import PyPopulation
from .phenotype import Phenotype


class Population:
    """
    Represents a population in a genetic algorithm.
    """

    def __init__(self, individuals: List[Phenotype] | List[Genotype] | PyPopulation):
        """
        Initializes a Population instance.

        :param individuals: A list of Phenotype instances.
        """
        if isinstance(individuals, PyPopulation):
            self.__inner = individuals
        elif isinstance(individuals, list):
            if all(isinstance(ind, Genotype) for ind in individuals):
                individuals = [Phenotype(genotype=ind) for ind in individuals]
            if not all(isinstance(ind, Phenotype) for ind in individuals):
                raise ValueError("All individuals must be instances of Phenotype")
            self.__inner = PyPopulation([ind.py_phenotype() for ind in individuals])
        else:
            raise TypeError(
                "individuals must be a list of Phenotype instances or a PyPopulation instance"
            )

    def __repr__(self):
        return f"Population(individuals={self.__inner})"

    def __len__(self):
        """
        Returns the number of individuals in the population.
        :return: Number of individuals in the population.
        """
        return len(self.__inner)

    def __iter__(self):
        """
        Returns an iterator over the individuals in the population.
        :return: An iterator over the individuals in the population.
        """
        for phenotype in self.__inner.phenotypes:
            yield Phenotype(phenotype=phenotype)

    def py_population(self) -> PyPopulation:
        """
        Returns the underlying PyPopulation instance.
        :return: The PyPopulation instance associated with this Population.
        """
        return self.__inner

    def phenotypes(self) -> List[Phenotype]:
        """
        Returns the phenotypes in the population.
        :return: A list of Phenotype instances.
        """
        return [Phenotype(phenotype=phenotype) for phenotype in self.__inner.phenotypes]
