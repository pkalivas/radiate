import pytest

from radiate import (
    PermutationCodec,
)


class TestPermutationCodec:
    """Unit tests for PermutationCodec."""

    @pytest.mark.unit
    def test_permutation_codec_creation(self):
        """Test creating a permutation codec."""
        codec = PermutationCodec([0, 1, 2, 3, 4])
        genotype = codec.encode()

        assert len(genotype) == 1
        assert len(genotype[0]) == 5

        # Check that it's a valid permutation
        decoded = codec.decode(genotype)
        assert len(decoded) == 5
        assert len(set(decoded)) == 5  # All unique values
        assert all(0 <= x < 5 for x in decoded)  # Values in range

    @pytest.mark.unit
    def test_permutation_codec_custom_values(self):
        """Test permutation codec with custom values."""
        values = [10, 20, 30, 40, 50]
        codec = PermutationCodec(values)
        genotype = codec.encode()
        decoded = codec.decode(genotype)

        assert len(decoded) == 5
        assert set(decoded) == set(values)  # Same values, different order
