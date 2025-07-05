import pytest
from radiate import BitCodec


class TestBitCodec:
    """Unit tests for BitCodec."""

    @pytest.mark.unit
    def test_bit_codec_vector_creation(self):
        """Test creating a bit codec for vectors."""
        codec = BitCodec.vector(8)
        genotype = codec.encode()

        assert len(genotype) == 1
        assert len(genotype[0]) == 8
        assert all(isinstance(gene.allele(), bool) for gene in genotype[0])

    @pytest.mark.unit
    def test_bit_codec_matrix_creation(self):
        """Test creating a bit codec for matrices."""
        codec = BitCodec.matrix((3, 4))
        genotype = codec.encode()

        assert len(genotype) == 3
        assert all(len(row) == 4 for row in genotype)
        assert all(isinstance(gene.allele(), bool) for row in genotype for gene in row)

    @pytest.mark.unit
    def test_bit_codec_decode(self):
        """Test decoding bit genotypes."""
        codec = BitCodec.vector(5)
        genotype = codec.encode()
        decoded = codec.decode(genotype)

        assert len(decoded) == 5
        assert all(isinstance(x, bool) for x in decoded)

    @pytest.mark.unit
    def test_empty_matrix_codec(self):
        """Test matrix codecs handle zero dimensions gracefully."""
        with pytest.raises(ValueError):
            BitCodec.matrix((3, 0))
