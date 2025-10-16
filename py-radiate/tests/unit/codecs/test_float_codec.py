import pytest
import numpy as np

import radiate as rd
from radiate import (
    FloatCodec,
)


class TestFloatCodec:
    """Unit tests for FloatCodec."""

    @pytest.mark.unit
    def test_float_codec_vector_creation(self):
        """Test creating a float codec for vectors."""
        codec = FloatCodec.vector(length=4, init_range=(-1.0, 1.0))
        genotype = codec.encode()

        assert len(genotype) == 1
        assert len(genotype[0]) == 4
        assert all(-1.0 <= gene.allele() <= 1.0 for gene in genotype[0])

    @pytest.mark.unit
    def test_float_codec_matrix_creation(self):
        """Test creating a float codec for matrices."""
        codec = FloatCodec.matrix((2, 3), init_range=(-10.0, 10.0))
        genotype = codec.encode()

        assert len(genotype) == 2
        assert all(len(row) == 3 for row in genotype)
        assert all(-10.0 <= gene.allele() <= 10.0 for row in genotype for gene in row)

    @pytest.mark.unit
    def test_float_codec_decode(self):
        """Test decoding float genotypes."""
        codec = FloatCodec.vector(length=3, init_range=(0.0, 1.0), use_numpy=True)
        genotype = codec.encode()
        decoded = codec.decode(genotype)

        assert len(decoded) == 3
        assert all(isinstance(x, np.float32) for x in decoded)
        assert all(0.0 <= x <= 1.0 for x in decoded)

    @pytest.mark.unit
    def test_float_codec_with_numpy(self):
        """Test float codec with numpy arrays."""
        codec = FloatCodec.vector(length=3, init_range=(-1.0, 1.0), use_numpy=True)
        genotype = codec.encode()
        decoded = codec.decode(genotype)

        assert isinstance(decoded, np.ndarray)
        assert decoded.shape == (3,)
        assert all(-1.0 <= x <= 1.0 for x in decoded)

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_shape(self):
        """Test FloatCodec matrix with invalid shape."""
        with pytest.raises(
            ValueError, match="Shape must be a tuple of \\(rows, cols\\)"
        ):
            FloatCodec.matrix(shape=(1, 2, 3))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_value_range(self):
        """Test FloatCodec matrix with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            FloatCodec.matrix(shape=(2, 3), init_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_value_range_order(self):
        """Test FloatCodec matrix with invalid value range order"""
        with pytest.raises(
            ValueError, match="Minimum value must be less than maximum value"
        ):
            FloatCodec.matrix(shape=(2, 3), init_range=(10.0, 5.0))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_bound_range(self):
        """Test FloatCodec matrix with invalid bound range (lines 82, 84-87)."""
        with pytest.raises(
            ValueError, match="Bound range must be a tuple of \\(min, max\\)"
        ):
            FloatCodec.matrix(shape=(2, 3), bounds=(1.0,))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_bound_range_order(self):
        """Test FloatCodec matrix with invalid bound range order."""
        with pytest.raises(
            ValueError, match="Minimum bound must be less than maximum bound"
        ):
            FloatCodec.matrix(shape=(2, 3), bounds=(10.0, 5.0))

    @pytest.mark.unit
    def test_float_codec_vector_invalid_length(self):
        """Test FloatCodec vector with invalid length."""
        with pytest.raises(ValueError, match="Length must be a positive integer"):
            FloatCodec.vector(length=0)

    @pytest.mark.unit
    def test_float_codec_vector_invalid_value_range(self):
        """Test FloatCodec vector with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            FloatCodec.vector(length=5, init_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_vector_invalid_bound_range(self):
        """Test FloatCodec vector with invalid bound range."""
        with pytest.raises(
            ValueError, match="Bound range must be a tuple of \\(min, max\\)"
        ):
            FloatCodec.vector(length=5, bounds=(1.0,))

    @pytest.mark.unit
    def test_float_codec_scalar_invalid_value_range(self):
        """Test FloatCodec scalar with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            FloatCodec.scalar(init_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_scalar_invalid_bound_range(self):
        """Test FloatCodec scalar with invalid bound range."""
        with pytest.raises(
            ValueError, match="Bound range must be a tuple of \\(min, max\\)"
        ):
            FloatCodec.scalar(bounds=(1.0,))

    @pytest.mark.unit
    def test_negative_length_codec(self):
        """Test codecs handle negative length gracefully."""
        with pytest.raises(ValueError):
            FloatCodec.vector(length=-1, init_range=(-1.0, 1.0))

    @pytest.mark.unit
    def test_codec_from_genes(self):
        """Test codec creation from genes."""
        genes = [rd.gene.float(0.5), rd.gene.float(0.8)]
        codec = FloatCodec(genes)
        assert isinstance(codec, FloatCodec)
        assert codec.decode(codec.encode()) == [genes[0].allele(), genes[1].allele()]
