import pytest
from radiate import BitCodec


@pytest.mark.unit
def test_bit_codec_vector_creation():
    """Test creating a bit codec for vectors."""
    codec = BitCodec(8)
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 8
    assert all(isinstance(gene.allele(), bool) for gene in genotype[0])

    codec = BitCodec(shape=12)
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 12
    assert all(isinstance(gene.allele(), bool) for gene in genotype[0])


@pytest.mark.unit
def test_bit_codec_matrix_creation():
    """Test creating a bit codec for matrices."""
    codec = BitCodec((3, 4))
    genotype = codec.encode()

    assert len(genotype) == 3
    assert all(len(row) == 4 for row in genotype)
    assert all(isinstance(gene.allele(), bool) for row in genotype for gene in row)

    other_codec = BitCodec(shape=[2, 3, 2])
    other_genotype = other_codec.encode()

    assert len(other_genotype) == 3
    assert len(other_genotype[0]) == 2
    assert len(other_genotype[1]) == 3
    assert len(other_genotype[2]) == 2

    other_decoded = other_codec.decode(other_genotype)
    assert len(other_decoded) == 3
    assert len(other_decoded[0]) == 2
    assert len(other_decoded[1]) == 3
    assert len(other_decoded[2]) == 2

    for row in other_genotype:
        for gene in row:
            assert isinstance(gene.allele(), bool)


@pytest.mark.unit
def test_bit_codec_decode():
    """Test decoding bit genotypes."""
    codec = BitCodec(5)
    genotype = codec.encode()
    decoded = codec.decode(genotype)

    assert len(decoded) == 5
    assert all(isinstance(x, bool) for x in decoded)

    codec = BitCodec(shape=(2, 3))
    genotype = codec.encode()
    decoded = codec.decode(genotype)

    assert len(decoded) == 2
    assert all(len(row) == 3 for row in decoded)
    assert all(isinstance(x, bool) for row in decoded for x in row)


@pytest.mark.unit
def test_empty_matrix_codec():
    """Test matrix codecs handle zero dimensions gracefully."""
    with pytest.raises(ValueError):
        BitCodec((3, 0))

    with pytest.raises(ValueError):
        BitCodec((0, 4))
