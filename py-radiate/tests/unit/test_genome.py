import pytest
import radiate as rd
from radiate.genome import Population, Phenotype, Genotype, GeneType


class TestPopulation:
    """Comprehensive tests for Population class to cover missing lines."""

    @pytest.mark.unit
    def test_population_creation_with_py_population(self):
        """Test Population creation with PyPopulation instance (line 19-26)."""
        chromosome = rd.chromosome.int(length=3, init_range=(0, 10))
        genotype = rd.Genotype([chromosome])
        phenotype = rd.Phenotype(genotype)
        population = rd.Population([phenotype])

        assert len(population) == 1
        assert isinstance(population, Population)

    @pytest.mark.unit
    def test_population_iteration(self):
        """Test Population iteration."""
        chromosome = rd.chromosome.int(length=3, init_range=(0, 10))
        genotype = rd.Genotype(chromosome)
        phenotype = rd.Phenotype(genotype)

        population = Population([phenotype])
        individuals = list(population)

        assert len(individuals) == 1
        assert isinstance(individuals[0], Phenotype)
        assert individuals[0] == phenotype

    @pytest.mark.unit
    def test_population_phenotypes_method(self):
        """Test Population phenotypes method"""
        chromosome = rd.chromosome.int(length=3, init_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        population = Population([phenotype])
        phenotypes = list(population)

        assert len(phenotypes) == 1
        assert isinstance(phenotypes[0], Phenotype)
        assert phenotypes[0] == phenotype

    @pytest.mark.unit
    def test_population_all_items_have_correct_gene_type(self):
        """Test that all items in the population have the correct gene type."""
        num_genes = 3
        num_chromosomes = 5
        num_phenotypes = 10

        population = Population(
            Phenotype(
                Genotype(
                    rd.chromosome.int(num_genes, init_range=(0, 10))
                    for _ in range(num_chromosomes)
                )
            )
            for _ in range(num_phenotypes)
        )

        assert len(population) == num_phenotypes
        assert isinstance(population, Population)
        assert population.gene_type() == GeneType.INT

        for individual in population:
            assert individual.gene_type() == GeneType.INT
            assert isinstance(individual, Phenotype)
            assert isinstance(individual.genotype(), Genotype)
            assert individual.genotype().gene_type() == GeneType.INT

            for chromosome in individual.genotype():
                assert isinstance(chromosome, rd.Chromosome)
                assert chromosome.gene_type() == GeneType.INT
                for gene in chromosome:
                    assert isinstance(gene, rd.Gene)
                    assert gene.allele() >= 0 and gene.allele() <= 10


class TestPhenotypes:
    """Comprehensive tests for Phenotype class to cover missing lines."""

    @pytest.mark.unit
    def test_phenotype_score_method(self):
        """Test Phenotype score method."""
        chromosome = rd.chromosome.int(length=3, init_range=(0, 10))
        genotype = rd.Genotype([chromosome])
        phenotype = rd.Phenotype(genotype)

        score = phenotype.score()
        assert isinstance(score, list)

    @pytest.mark.unit
    def test_phenotype_genotype_method(self):
        """Test Phenotype genotype method."""
        chromosome = rd.chromosome.int(length=3, init_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype)

        # Test genotype method
        retrieved_genotype = phenotype.genotype()
        assert isinstance(retrieved_genotype, Genotype)
        assert retrieved_genotype == genotype


class TestChromosomes:
    @pytest.mark.unit
    def test_float_chromosome_creation(self):
        chromosome = rd.chromosome.float(length=5, init_range=(-10.0, 10.0))

        assert len(chromosome) == 5
        for gene in chromosome:
            assert isinstance(gene.allele(), float)
            assert gene.allele() >= -10.0 and gene.allele() <= 10.0

    @pytest.mark.unit
    def test_int_chromosome_creation(self):
        chromosome = rd.chromosome.int(length=5, init_range=(0, 10))

        assert len(chromosome) == 5
        for gene in chromosome:
            assert isinstance(gene.allele(), int)
            assert gene.allele() >= 0 and gene.allele() <= 10

    @pytest.mark.unit
    def test_char_chromosome_creation(self):
        chromosome = rd.chromosome.char(length=5, char_set={"a", "b", "c"})

        assert len(chromosome) == 5
        for gene in chromosome:
            assert isinstance(gene.allele(), str)
            assert gene.allele() in {"a", "b", "c"}

    @pytest.mark.unit
    def test_bit_chromosome_creation(self):
        chromosome = rd.chromosome.bit(length=5)

        assert len(chromosome) == 5
        for gene in chromosome:
            assert isinstance(gene.allele(), bool)
            assert gene.allele() in {True, False}

    @pytest.mark.unit
    def test_genotype_creation_from_chromosomes(self):
        chromosome1 = rd.chromosome.int(length=3, init_range=(0, 10))
        chromosome2 = rd.chromosome.int(length=4, init_range=(0, 5))

        genotype = rd.Genotype([chromosome1, chromosome2])

        assert len(genotype) == 2
        assert genotype[0] == chromosome1
        assert genotype[1] == chromosome2


class TestGenes:
    @pytest.mark.unit
    def test_float_gene_creation(self):
        gene = rd.gene.float(init_range=(-10.0, 10.0))

        assert isinstance(gene.allele(), float)
        assert gene.allele() is not None
        assert gene.allele() >= -10.0 and gene.allele() <= 10.0

    @pytest.mark.unit
    def test_int_gene_creation(self):
        gene = rd.gene.int(init_range=(0, 10))

        assert isinstance(gene.allele(), int)
        assert gene.allele() is not None
        assert gene.allele() >= 0 and gene.allele() <= 10

    @pytest.mark.unit
    def test_char_gene_creation(self):
        gene = rd.gene.char(char_set={"a", "b", "c"})

        assert isinstance(gene.allele(), str)
        assert gene.allele() is not None
        assert gene.allele() in {"a", "b", "c"}

    @pytest.mark.unit
    def test_bit_gene_creation(self):
        gene = rd.gene.bit()

        assert isinstance(gene.allele(), bool)
        assert gene.allele() is not None
        assert gene.allele() in {True, False}
