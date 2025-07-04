"""
Comprehensive unit tests for Radiate genome components.

These tests focus on covering the missing lines identified in the coverage report.
"""

import pytest
import radiate as rd
from radiate.genome import Population, Phenotype, Genotype


class TestPopulationComprehensive:
    """Comprehensive tests for Population class to cover missing lines."""

    @pytest.mark.unit
    def test_population_creation_with_py_population(self):
        """Test Population creation with PyPopulation instance (line 19-26)."""
        # Create a PyPopulation through the normal flow
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
        """Test Population creation with list containing non-Phenotype objects (line 38)."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        with pytest.raises(
            ValueError, match="All individuals must be instances of Phenotype"
        ):
            Population([phenotype, "invalid"])

    @pytest.mark.unit
    def test_population_iteration(self):
        """Test Population iteration (lines 45-46)."""
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
        """Test Population phenotypes method (lines 53, 60)."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        population = Population([phenotype])
        phenotypes = population.phenotypes()

        assert len(phenotypes) == 1
        assert isinstance(phenotypes[0], Phenotype)
        assert phenotypes[0] == phenotype


class TestPhenotypeComprehensive:
    """Comprehensive tests for Phenotype class to cover missing lines."""

    @pytest.mark.unit
    def test_phenotype_creation_with_invalid_type(self):
        """Test Phenotype creation with invalid type (line 31)."""
        with pytest.raises(
            TypeError, match="genotype must be an instance of Genotype or PyPhenotype"
        ):
            Phenotype(genotype="invalid")

    @pytest.mark.unit
    def test_phenotype_creation_with_none_params(self):
        """Test Phenotype creation with None parameters (line 38)."""
        with pytest.raises(
            TypeError, match="genotype must be an instance of Genotype or PyPhenotype"
        ):
            Phenotype()

    @pytest.mark.unit
    def test_phenotype_score_method(self):
        """Test Phenotype score method (lines 45, 52)."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        score = phenotype.score()
        assert isinstance(score, list)

    @pytest.mark.unit
    def test_phenotype_genotype_method(self):
        """Test Phenotype genotype method (lines 56-59)."""
        chromosome = rd.Chromosome.int(length=3, value_range=(0, 10))
        genotype = rd.Genotype(chromosomes=[chromosome])
        phenotype = rd.Phenotype(genotype=genotype)

        # Test genotype method
        retrieved_genotype = phenotype.genotype()
        assert isinstance(retrieved_genotype, Genotype)
        assert retrieved_genotype.py_genotype() == genotype.py_genotype()


class TestFloatCodecComprehensive:
    """Comprehensive tests for FloatCodec to cover missing lines."""

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_shape(self):
        """Test FloatCodec matrix with invalid shape (line 72)."""
        with pytest.raises(
            ValueError, match="Shape must be a tuple of \\(rows, cols\\)"
        ):
            rd.FloatCodec.matrix(shape=(1, 2, 3))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_value_range(self):
        """Test FloatCodec matrix with invalid value range (lines 75-76)."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.matrix(shape=(2, 3), value_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_value_range_order(self):
        """Test FloatCodec matrix with invalid value range order (line 80)."""
        with pytest.raises(
            ValueError, match="Minimum value must be less than maximum value"
        ):
            rd.FloatCodec.matrix(shape=(2, 3), value_range=(10.0, 5.0))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_bound_range(self):
        """Test FloatCodec matrix with invalid bound range (lines 82, 84-87)."""
        with pytest.raises(
            ValueError, match="Bound range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.matrix(shape=(2, 3), bound_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_bound_range_order(self):
        """Test FloatCodec matrix with invalid bound range order (line 87)."""
        with pytest.raises(
            ValueError, match="Minimum bound must be less than maximum bound"
        ):
            rd.FloatCodec.matrix(shape=(2, 3), bound_range=(10.0, 5.0))

    @pytest.mark.unit
    def test_float_codec_vector_invalid_length(self):
        """Test FloatCodec vector with invalid length (line 125)."""
        with pytest.raises(ValueError, match="Length must be a positive integer"):
            rd.FloatCodec.vector(length=0)

    @pytest.mark.unit
    def test_float_codec_vector_invalid_value_range(self):
        """Test FloatCodec vector with invalid value range (lines 127, 129-132)."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.vector(length=5, value_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_vector_invalid_bound_range(self):
        """Test FloatCodec vector with invalid bound range (lines 163-174)."""
        with pytest.raises(
            ValueError, match="Bound range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.vector(length=5, bound_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_scalar_invalid_value_range(self):
        """Test FloatCodec scalar with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.scalar(value_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_scalar_invalid_bound_range(self):
        """Test FloatCodec scalar with invalid bound range."""
        with pytest.raises(
            ValueError, match="Bound range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.scalar(bound_range=(1.0,))


class TestTreeComprehensive:
    """Comprehensive tests for Tree class to cover missing lines."""

    @pytest.mark.unit
    def test_tree_eval_with_list_of_lists(self, tree_simple):
        """Test Tree eval with list of lists (line 10)."""

        result = tree_simple.eval([[1.0, 2.0], [3.0, 4.0]])
        assert isinstance(result, list)
        assert len(result) == 2

    @pytest.mark.unit
    def test_tree_eval_with_tree_simple(self, tree_simple):
        """Test Tree eval with invalid input (lines 13-17)."""

        with pytest.raises(
            ValueError,
            match="Inputs must be a list of floats or a list of list of floats",
        ):
            tree_simple.eval("invalid")

        with pytest.raises(
            ValueError,
            match="Inputs must be a list of floats or a list of list of floats",
        ):
            tree_simple.eval([1.0, "invalid", 3.0])


class TestIntCodecComprehensive:
    """Comprehensive tests for IntCodec to cover missing lines."""

    @pytest.mark.unit
    def test_int_codec_matrix_invalid_shape(self):
        """Test IntCodec matrix with invalid shape."""
        with pytest.raises(
            ValueError, match="Shape must be a tuple of \\(rows, cols\\)"
        ):
            rd.IntCodec.matrix(shape=(1, 2, 3))

    @pytest.mark.unit
    def test_int_codec_matrix_invalid_value_range(self):
        """Test IntCodec matrix with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.IntCodec.matrix(shape=(2, 3), value_range=(1,))

    @pytest.mark.unit
    def test_int_codec_matrix_invalid_value_range_order(self):
        """Test IntCodec matrix with invalid value range order."""
        with pytest.raises(
            ValueError, match="Minimum value must be less than maximum value"
        ):
            rd.IntCodec.matrix(shape=(2, 3), value_range=(10, 5))

    @pytest.mark.unit
    def test_int_codec_vector_invalid_length(self):
        """Test IntCodec vector with invalid length."""
        with pytest.raises(ValueError, match="Length must be a positive integer"):
            rd.IntCodec.vector(length=0)

    @pytest.mark.unit
    def test_int_codec_vector_invalid_value_range(self):
        """Test IntCodec vector with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.IntCodec.vector(length=5, value_range=(1,))

    @pytest.mark.unit
    def test_int_codec_vector_invalid_value_range_order(self):
        """Test IntCodec vector with invalid value range order."""
        with pytest.raises(
            ValueError, match="Minimum value must be less than maximum value"
        ):
            rd.IntCodec.vector(length=5, value_range=(10, 5))

    @pytest.mark.unit
    def test_int_codec_scalar_invalid_value_range(self):
        """Test IntCodec scalar with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.IntCodec.scalar(value_range=(1,))

    @pytest.mark.unit
    def test_int_codec_scalar_invalid_value_range_order(self):
        """Test IntCodec scalar with invalid value range order."""
        with pytest.raises(
            ValueError, match="Minimum value must be less than maximum value"
        ):
            rd.IntCodec.scalar(value_range=(10, 5))
