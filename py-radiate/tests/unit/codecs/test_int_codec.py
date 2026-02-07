import pytest

from radiate import IntCodec


@pytest.mark.unit
def test_int_codec_vector_creation():
    """Test creating an integer codec for vectors."""
    codec = IntCodec.vector(length=5, init_range=(0, 10))
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 5
    assert all(0 <= gene.allele() <= 10 for gene in genotype[0])

    codec = IntCodec(shape=8, init_range=(-5, 5))
    genotype = codec.encode()

    assert len(genotype) == 1
    assert len(genotype[0]) == 8
    assert all(-5 <= gene.allele() <= 5 for gene in genotype[0])


@pytest.mark.unit
def test_int_codec_matrix_creation():
    """Test creating an integer codec for matrices."""
    codec = IntCodec.matrix((3, 4), init_range=(0, 100))
    genotype = codec.encode()

    assert len(genotype) == 3
    assert all(len(row) == 4 for row in genotype)
    for row in genotype:
        assert all(isinstance(gene.allele(), int) for gene in row)
    assert all(0 <= gene.allele() <= 100 for row in genotype for gene in row)

    codec = IntCodec(shape=(2, 3), init_range=(-10, 10))
    genotype = codec.encode()

    assert len(genotype) == 2
    assert all(len(row) == 3 for row in genotype)
    for row in genotype:
        assert all(isinstance(gene.allele(), int) for gene in row)
    assert all(-10 <= gene.allele() <= 10 for row in genotype for gene in row)


@pytest.mark.unit
def test_int_codec_decode():
    """Test decoding integer genotypes."""
    codec = IntCodec.vector(length=3, init_range=(0, 5))
    genotype = codec.encode()
    decoded = codec.decode(genotype)

    assert len(decoded) == 3
    assert all(isinstance(x, int) for x in decoded)
    assert all(0 <= x <= 5 for x in decoded)

    codec = IntCodec(shape=(2, 2), init_range=(-5, 5))
    genotype = codec.encode()
    decoded = codec.decode(genotype)

    assert len(decoded) == 2
    assert all(len(row) == 2 for row in decoded)
    assert all(isinstance(x, int) for row in decoded for x in row)
    assert all(-5 <= x <= 5 for row in decoded for x in row)


@pytest.mark.unit
def test_int_codec_bounds():
    """Test integer codec respects bounds."""
    codec = IntCodec.vector(length=10, init_range=(-5, 5))
    genotype = codec.encode()

    for gene in genotype[0]:
        assert -5 <= gene.allele() <= 5

    codec = IntCodec.matrix((3, 3), init_range=(0, 100))
    genotype = codec.encode()

    for row in genotype:
        for gene in row:
            assert 0 <= gene.allele() <= 100


@pytest.mark.unit
def test_int_codec_invalid_bounds():
    """Test integer codec handles invalid bounds."""
    with pytest.raises(ValueError):
        IntCodec.vector(length=5, init_range=(10, 5))  # min > max


@pytest.mark.unit
def test_zero_length_codec():
    """Test codecs handle zero length gracefully."""
    with pytest.raises(ValueError):
        IntCodec.vector(length=0, init_range=(0, 10))
