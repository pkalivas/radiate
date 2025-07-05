"""
Tests for Radiate genome components.

These tests focus on covering the missing lines identified in the coverage report.
"""

import pytest
import radiate as rd
from radiate.genome import Population, Phenotype, Genotype


class TestPopulation:
    """Comprehensive tests for Population class to cover missing lines."""

    @pytest.mark.unit
    def test_population_creation_with_py_population(self):
        """Test Population creation with PyPopulation instance (line 19-26)."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)
        population = rd.Population([phenotype])

        assert len(population) == 1
        assert isinstance(population, Population)

    @pytest.mark.unit
    def test_population_creation_with_invalid_type(self):
        """Test Population creation with invalid type (line 31)."""
        with pytest.raises(
            TypeError,
            match="individuals must be a list of Phenotype instances or a PyPopulation instance",
        ):
            Population("invalid")

    @pytest.mark.unit
    def test_population_creation_with_mixed_list(self):
        """Test Population creation with list containing non-Phenotype objects."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        with pytest.raises(
            ValueError, match="All individuals must be instances of Phenotype"
        ):
            Population([phenotype, "invalid"])

    @pytest.mark.unit
    def test_population_iteration(self):
        """Test Population iteration."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        population = Population([phenotype])
        individuals = list(population)

        assert len(individuals) == 1
        assert isinstance(individuals[0], Phenotype)
        assert individuals[0] == phenotype

    @pytest.mark.unit
    def test_population_phenotypes_method(self):
        """Test Population phenotypes method"""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        population = Population([phenotype])
        phenotypes = population.phenotypes()

        assert len(phenotypes) == 1
        assert isinstance(phenotypes[0], Phenotype)
        assert phenotypes[0] == phenotype


class TestPhenotypes:
    """Comprehensive tests for Phenotype class to cover missing lines."""

    @pytest.mark.unit
    def test_phenotype_creation_with_invalid_type(self):
        """Test Phenotype creation with invalid type"""
        with pytest.raises(
            TypeError, match="genotype must be an instance of Genotype or PyPhenotype"
        ):
            Phenotype(genotype="invalid")

    @pytest.mark.unit
    def test_phenotype_creation_with_none_params(self):
        """Test Phenotype creation with None parameters"""
        with pytest.raises(
            TypeError, match="genotype must be an instance of Genotype or PyPhenotype"
        ):
            Phenotype()

    @pytest.mark.unit
    def test_phenotype_score_method(self):
        """Test Phenotype score method."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        score = phenotype.score()
        assert isinstance(score, list)

    @pytest.mark.unit
    def test_phenotype_genotype_method(self):
        """Test Phenotype genotype method."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        # Test genotype method
        retrieved_genotype = phenotype.genotype()
        assert isinstance(retrieved_genotype, Genotype)
        assert retrieved_genotype.py_genotype() == genotype.py_genotype()


class TestChromosomes:
    @pytest.mark.unit
    def test_float_chromosome_creation(self):
        chromosome = rd.Chromosome.float(length=5, value_range=(-10.0, 10.0))

        assert len(chromosome.genes()) == 5
        for gene in chromosome.genes():
            assert isinstance(gene.allele(), float)
            assert gene.allele() >= -10.0 and gene.allele() <= 10.0

    @pytest.mark.unit
    def test_int_chromosome_creation(self):
        chromosome = rd.Chromosome.int(length=5, value_range=(0, 10))

        assert len(chromosome.genes()) == 5
        for gene in chromosome.genes():
            assert isinstance(gene.allele(), int)
            assert gene.allele() >= 0 and gene.allele() <= 10

    @pytest.mark.unit
    def test_char_chromosome_creation(self):
        chromosome = rd.Chromosome.char(length=5, char_set={"a", "b", "c"})

        assert len(chromosome.genes()) == 5
        for gene in chromosome.genes():
            assert isinstance(gene.allele(), str)
            assert gene.allele() in {"a", "b", "c"}

    @pytest.mark.unit
    def test_bit_chromosome_creation(self):
        chromosome = rd.Chromosome.bit(length=5)

        assert len(chromosome.genes()) == 5
        for gene in chromosome.genes():
            assert isinstance(gene.allele(), bool)
            assert gene.allele() in {True, False}

    @pytest.mark.unit
    def test_genotype_creation_from_chromosomes(self):
        chromosome1 = rd.Chromosome.int(length=3, value_range=(0, 10))
        chromosome2 = rd.Chromosome.int(length=4, value_range=(0, 5))

        genotype = rd.Genotype(chromosomes=[chromosome1, chromosome2])

        assert len(genotype) == 2
        assert genotype.chromosomes()[0] == chromosome1
        assert genotype.chromosomes()[1] == chromosome2


class TestGenes:
    @pytest.mark.unit
    def test_float_gene_creation(self):
        gene = rd.Gene.float(value_range=(-10.0, 10.0))

        assert isinstance(gene.allele(), float)
        assert gene.allele() is not None
        assert gene.allele() >= -10.0 and gene.allele() <= 10.0

    @pytest.mark.unit
    def test_int_gene_creation(self):
        gene = rd.Gene.int(value_range=(0, 10))

        assert isinstance(gene.allele(), int)
        assert gene.allele() is not None
        assert gene.allele() >= 0 and gene.allele() <= 10

    @pytest.mark.unit
    def test_char_gene_creation(self):
        gene = rd.Gene.char(char_set={"a", "b", "c"})

        assert isinstance(gene.allele(), str)
        assert gene.allele() is not None
        assert gene.allele() in {"a", "b", "c"}

    @pytest.mark.unit
    def test_bit_gene_creation(self):
        gene = rd.Gene.bit()

        assert isinstance(gene.allele(), bool)
        assert gene.allele() is not None
        assert gene.allele() in {True, False}
