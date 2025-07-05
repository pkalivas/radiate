import pytest
import numpy as np

try:
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False
    np = None

import radiate as rd
from radiate import (
    FloatCodec,
)


class TestFloatCodec:
    """Unit tests for FloatCodec."""

    @pytest.mark.unit
    def test_float_codec_vector_creation(self):
        """Test creating a float codec for vectors."""
        codec = FloatCodec.vector(length=4, value_range=(-1.0, 1.0))
        genotype = codec.encode()

        assert len(genotype) == 1
        assert len(genotype[0]) == 4
        assert all(-1.0 <= gene.allele() <= 1.0 for gene in genotype[0])

    @pytest.mark.unit
    def test_float_codec_matrix_creation(self):
        """Test creating a float codec for matrices."""
        codec = FloatCodec.matrix((2, 3), value_range=(-10.0, 10.0))
        genotype = codec.encode()

        assert len(genotype) == 2
        assert all(len(row) == 3 for row in genotype)
        assert all(-10.0 <= gene.allele() <= 10.0 for row in genotype for gene in row)

    @pytest.mark.unit
    def test_float_codec_decode(self):
        """Test decoding float genotypes."""
        codec = FloatCodec.vector(length=3, value_range=(0.0, 1.0))
        genotype = codec.encode()
        decoded = codec.decode(genotype)

        assert len(decoded) == 3
        assert all(isinstance(x, float) for x in decoded)
        assert all(0.0 <= x <= 1.0 for x in decoded)

    @pytest.mark.unit
    def test_float_codec_with_numpy(self):
        """Test float codec with numpy arrays."""
        codec = FloatCodec.vector(length=3, value_range=(-1.0, 1.0), use_numpy=True)
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
            rd.FloatCodec.matrix(shape=(1, 2, 3))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_value_range(self):
        """Test FloatCodec matrix with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.matrix(shape=(2, 3), value_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_matrix_invalid_value_range_order(self):
        """Test FloatCodec matrix with invalid value range order"""
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
        """Test FloatCodec matrix with invalid bound range order."""
        with pytest.raises(
            ValueError, match="Minimum bound must be less than maximum bound"
        ):
            rd.FloatCodec.matrix(shape=(2, 3), bound_range=(10.0, 5.0))

    @pytest.mark.unit
    def test_float_codec_vector_invalid_length(self):
        """Test FloatCodec vector with invalid length."""
        with pytest.raises(ValueError, match="Length must be a positive integer"):
            rd.FloatCodec.vector(length=0)

    @pytest.mark.unit
    def test_float_codec_vector_invalid_value_range(self):
        """Test FloatCodec vector with invalid value range."""
        with pytest.raises(
            ValueError, match="Value range must be a tuple of \\(min, max\\)"
        ):
            rd.FloatCodec.vector(length=5, value_range=(1.0,))

    @pytest.mark.unit
    def test_float_codec_vector_invalid_bound_range(self):
        """Test FloatCodec vector with invalid bound range."""
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

    @pytest.mark.unit
    def test_negative_length_codec(self):
        """Test codecs handle negative length gracefully."""
        with pytest.raises(ValueError):
            FloatCodec.vector(length=-1, value_range=(-1.0, 1.0))
