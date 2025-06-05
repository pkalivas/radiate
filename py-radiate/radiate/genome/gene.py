from __future__ import annotations

from typing import Tuple, List
from radiate.radiate import PyGene, PyChromosome, PyGenotype, PyPhenotype, PyPopulation


class Population:
    """
    Represents a population in a genetic algorithm.
    """

    def __init__(self, individuals: List[Phenotype] | PyPopulation):
        """
        Initializes a Population instance.

        :param individuals: A list of Phenotype instances.
        """
        if isinstance(individuals, PyPopulation):
            self.__inner = individuals
        elif isinstance(individuals, list):
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


class Genotype:
    """
    Represents a genotype in a genome.
    """

    def __init__(
        self,
        genotype: PyGenotype | None = None,
        chromosomes: List[Chromosome] | Chromosome | None = None,
    ):
        """
        Initializes a Genotype instance.

        :param chromosomes: A list of Chromosome instances.
        """
        if genotype is not None:
            if isinstance(genotype, PyGenotype):
                self.__inner = genotype
            else:
                raise TypeError("genotype must be an instance of PyGenotype")
        elif chromosomes is not None:
            if isinstance(chromosomes, Chromosome):
                chromosomes = [chromosomes]
            if isinstance(chromosomes, list):
                if all(isinstance(chromo, Chromosome) for chromo in chromosomes):
                    self.__inner = PyGenotype(
                        [chromo.py_chromosome() for chromo in chromosomes]
                    )
                else:
                    raise ValueError("All chromosomes must be instances of Chromosome")
            else:
                raise TypeError(
                    "chromosomes must be a Chromosome instance or a list of Chromosome instances"
                )
        else:
            raise ValueError("Either genotype or chromosomes must be provided")

    def __repr__(self):
        return f"Genotype(chromosomes={self.__inner.chromosomes})"

    def __len__(self):
        """
        Returns the length of the genotype.
        :return: Length of the genotype.
        """
        return len(self.__inner.chromosomes)

    def py_genotype(self) -> PyGenotype:
        """
        Returns the underlying PyGenotype instance.
        :return: The PyGenotype instance associated with this Genotype.
        """
        return self.__inner

    def chromosomes(self) -> List[Chromosome]:
        """
        Returns the chromosomes of the genotype.
        :return: A list of Chromosome instances.
        """
        return [
            Chromosome(chromosome=chromosome) for chromosome in self.__inner.chromosomes
        ]


class Chromosome:
    """
    Represents a chromosome in a genome.
    """

    def __init__(
        self,
        chromosome: PyChromosome | None = None,
        genes: List[Gene] | Gene | None = None,
    ):
        """
        Initializes a Chromosome instance.

        :param gene_type: The type of the genes in the chromosome.
        :param length: The length of the chromosome.
        """
        if chromosome is not None:
            if isinstance(chromosome, PyChromosome):
                self.__inner = chromosome
            else:
                raise TypeError("chromosome must be an instance of PyChromosome")
        elif genes is not None:
            if isinstance(genes, Gene):
                self.__inner = PyChromosome([genes.py_gene()])
            if isinstance(genes, list):
                if all(isinstance(gene, Gene) for gene in genes):
                    self.__inner = PyChromosome([gene.py_gene() for gene in genes])
                else:
                    raise ValueError("All genes must be instances of Gene")
            else:
                raise TypeError(
                    "genes must be a Gene instance or a list of Gene instances"
                )
        else:
            raise ValueError("Either chromosome or genes must be provided")

    def __repr__(self):
        return f"Chromosome(genes={self.__inner.genes})"

    def __len__(self):
        """
        Returns the length of the chromosome.
        :return: Length of the chromosome.
        """
        return len(self.__inner.genes)

    def py_chromosome(self) -> PyChromosome:
        """
        Returns the underlying PyChromosome instance.
        :return: The PyChromosome instance associated with this Chromosome.
        """
        return self.__inner

    def genes(self) -> List[Gene]:
        """
        Returns the genes of the chromosome.
        :return: A list of Gene instances.
        """
        return [Gene(gene) for gene in self.__inner.genes]

    @staticmethod
    def float(
        length: int,
        value_range: Tuple[float, float] | None = None,
        bound_range: Tuple[float, float] | None = None,
    ) -> "Chromosome":
        """
        Create a float chromosome with specified length and optional parameters.
        :param length: Length of the chromosome.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Chromosome instance configured as a float chromosome.
        """
        genes = [
            Gene.float(value_range=value_range, bound_range=bound_range)
            for _ in range(length)
        ]
        return Chromosome(genes=genes)
    
    @staticmethod
    def int(
        length: int,
        value_range: Tuple[int, int] | None = None,
        bound_range: Tuple[int, int] | None = None,
    ) -> "Chromosome":
        """
        Create an integer chromosome with specified length and optional parameters.
        :param length: Length of the chromosome.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Chromosome instance configured as an integer chromosome.
        """
        genes = [
            Gene.int(value_range=value_range, bound_range=bound_range)
            for _ in range(length)
        ]
        return Chromosome(genes=genes)
    
    @staticmethod
    def bit(length: int) -> "Chromosome":
        """
        Create a bit chromosome with specified length and optional allele.
        :param length: Length of the chromosome.
        :param allele: Initial value of the gene.
        :return: A new Chromosome instance configured as a bit chromosome.
        """
        genes = [Gene.bit() for _ in range(length)]
        return Chromosome(genes=genes)
    
    @staticmethod
    def char(
        length: int,
        char_set: set[str] | None = None
    ) -> "Chromosome":
        """
        Create a character chromosome with specified length and optional character set.
        :param length: Length of the chromosome.
        :param char_set: Set of characters to choose from.
        :return: A new Chromosome instance configured as a character chromosome.
        """
        genes = [Gene.char(char_set=char_set) for _ in range(length)]
        return Chromosome(genes=genes)


class Gene:
    def __init__(self, py_gene: PyGene):
        """
        Initialize the Gene class.
        This class is a placeholder for gene-related functionality.
        """
        self.__inner = py_gene

    def __eq__(self, other: object) -> bool:
        """
        Check equality with another Gene instance.
        :param other: Another Gene instance to compare with.
        :return: True if both instances are equal, False otherwise.
        """
        if not isinstance(other, Gene):
            return False
        return self.__inner == other.__inner

    def __repr__(self) -> str:
        """
        Return a string representation of the Gene instance.
        :return: String representation of the Gene instance.
        """
        return f"Gene({self.__inner.__repr__()})"

    def __hash__(self) -> int:
        """
        Return a hash of the Gene instance.
        :return: Hash of the Gene instance.
        """
        return hash(self.__inner)

    def py_gene(self) -> PyGene:
        """
        Get the underlying PyGene instance.
        :return: The PyGene instance associated with this Gene.
        """
        return self.__inner

    @property
    def gene_type(self) -> str:
        """
        Get the type of the gene.
        :return: The type of the gene as a string.
        """
        return self.__inner.gene_type()

    @property
    def allele(self) -> float | int | bool | str | None:
        """
        Get the allele of the gene.
        :return: The allele of the gene, which can be a float, int, bool, str, or None.
        """
        return self.__inner.allele()

    @staticmethod
    def float(
        allele: float | None = None,
        value_range: Tuple[float, float] | None = None,
        bound_range: Tuple[float, float] | None = None,
    ) -> "Gene":
        """
        Create a float gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as a float gene.
        """
        return Gene(PyGene.float(allele=allele, range=value_range, bounds=bound_range))

    @staticmethod
    def int(
        allele: int | None = None,
        value_range: Tuple[int, int] | None = None,
        bound_range: Tuple[int, int] | None = None,
    ) -> "Gene":
        """
        Create an integer gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as an integer gene.
        """
        return Gene(PyGene.int(allele=allele, range=value_range, bounds=bound_range))

    @staticmethod
    def bit(allele: bool | None = None) -> "Gene":
        """
        Create a bit gene with an optional allele.
        :param allele: Initial value of the gene.
        :return: A new Gene instance configured as a bit gene.
        """
        return Gene(PyGene.bit(allele=allele))

    @staticmethod
    def char(allele: str | None = None, char_set: set[str] | None = None) -> "Gene":
        """
        Create a character gene with optional allele, value range, and bound range.
        :param allele: Initial value of the gene.
        :param value_range: Minimum and maximum value for the gene.
        :param bound_range: Minimum and maximum bound for the gene.
        :return: A new Gene instance configured as a character gene.
        """
        return Gene(
            PyGene.char(allele=allele, char_set=list(char_set) if char_set else None)
        )
