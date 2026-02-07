import pytest
from radiate import CharCodec, gene


@pytest.mark.unit
def test_char_codec_vector_creation():
    """Test creating a character codec for vectors."""
    codec = CharCodec.vector(length=5)
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 5
    assert all(isinstance(gene.allele(), str) for gene in genotype[0])


@pytest.mark.unit
def test_char_codec_matrix_creation():
    """Test creating a character codec for matrices."""
    codec = CharCodec.matrix(chromosomes=[3, 3], char_set="abc")
    genotype = codec.encode()

    assert len(genotype) == 2
    assert all(len(row) == 3 for row in genotype)
    assert all(isinstance(gene.allele(), str) for row in genotype for gene in row)


@pytest.mark.unit
def test_char_codec_from_genes():
    """Test creating a character codec from existing genes."""
    initial_genes = [gene.char("x"), gene.char("y"), gene.char("z")]
    codec = CharCodec.from_genes(initial_genes)
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 3
    assert all(initial_genes[i].allele() == genotype[0][i].allele() for i in range(3))
    assert all(isinstance(gene.allele(), str) for gene in genotype[0])


@pytest.mark.unit
def test_char_codec_decode():
    """Test decoding character genotypes."""
    codec = CharCodec.vector(length=4)
    genotype = codec.encode()
    decoded = codec.decode(genotype)

    assert len(decoded) == 4
    assert all(isinstance(x, str) and len(x) == 1 for x in decoded)


@pytest.mark.unit
def test_char_codec_custom_charset():
    """Test character codec with custom character set."""
    charset = "ABC"
    codec = CharCodec.vector(length=3, char_set=charset)
    genotype = codec.encode()

    for inner_gene in genotype[0]:
        assert inner_gene.allele() in charset


@pytest.mark.unit
def test_empty_matrix_codec():
    """Test matrix codecs handle zero dimensions gracefully."""
    with pytest.raises(ValueError):
        CharCodec.matrix((0, 3))
