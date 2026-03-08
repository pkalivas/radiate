import pytest
from radiate import CharCodec


@pytest.mark.unit
def test_char_codec_vector_creation():
    """Test creating a character codec for vectors."""
    codec = CharCodec(shape=5)
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 5
    assert all(isinstance(gene.allele(), str) for gene in genotype[0])

    codec = CharCodec(shape=10, char_set="abc")
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 10
    assert all(gene.allele() in "abc" for gene in genotype[0])


@pytest.mark.unit
def test_char_codec_matrix_creation():
    """Test creating a character codec for matrices."""
    codec = CharCodec(shape=[3, 3], char_set="abc")
    genotype = codec.encode()

    assert len(genotype) == 2
    assert all(len(row) == 3 for row in genotype)
    assert all(isinstance(gene.allele(), str) for row in genotype for gene in row)

    codec = CharCodec(shape=(4, 5), char_set="xyz")
    genotype = codec.encode()

    assert len(genotype) == 4
    assert all(len(row) == 5 for row in genotype)
    assert all(gene.allele() in "xyz" for row in genotype for gene in row)


@pytest.mark.unit
def test_char_codec_decode():
    """Test decoding character genotypes."""
    codec = CharCodec(shape=4)
    genotype = codec.encode()
    decoded = codec.decode(genotype)

    assert len(decoded) == 4
    assert all(isinstance(x, str) and len(x) == 1 for x in decoded)


@pytest.mark.unit
def test_char_codec_custom_charset():
    """Test character codec with custom character set."""
    charset = "ABC"
    codec = CharCodec(shape=3, char_set=charset)
    genotype = codec.encode()

    for inner_gene in genotype[0]:
        assert inner_gene.allele() in charset


@pytest.mark.unit
def test_empty_matrix_codec():
    """Test matrix codecs handle zero dimensions gracefully."""
    with pytest.raises(ValueError):
        CharCodec((0, 3))
