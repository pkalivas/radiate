import pytest
from radiate import (
    CharCodec,
)


class TestCharCodec:
    """Unit tests for CharCodec."""

    @pytest.mark.unit
    def test_char_codec_vector_creation(self):
        """Test creating a character codec for vectors."""
        codec = CharCodec.vector(length=5)
        genotype = codec.encode()

        assert len(genotype) == 1
        assert len(genotype[0]) == 5
        assert all(isinstance(gene.allele(), str) for gene in genotype[0])

    @pytest.mark.unit
    def test_char_codec_matrix_creation(self):
        """Test creating a character codec for matrices."""
        codec = CharCodec.matrix(chromosomes=[3, 3], char_set="abc")
        genotype = codec.encode()

        assert len(genotype) == 2
        assert all(len(row) == 3 for row in genotype)
        assert all(isinstance(gene.allele(), str) for row in genotype for gene in row)

    @pytest.mark.unit
    def test_char_codec_decode(self):
        """Test decoding character genotypes."""
        codec = CharCodec.vector(length=4)
        genotype = codec.encode()
        decoded = codec.decode(genotype)

        assert len(decoded) == 4
        assert all(isinstance(x, str) and len(x) == 1 for x in decoded)

    @pytest.mark.unit
    def test_char_codec_custom_charset(self):
        """Test character codec with custom character set."""
        charset = "ABC"
        codec = CharCodec.vector(length=3, char_set=charset)
        genotype = codec.encode()

        for gene in genotype[0]:
            assert gene.allele() in charset

    @pytest.mark.unit
    def test_empty_matrix_codec(self):
        """Test matrix codecs handle zero dimensions gracefully."""
        with pytest.raises(ValueError):
            CharCodec.matrix((0, 3))
