"""
Unit tests for Radiate codecs.

These tests focus on individual codec functionality and edge cases.
"""

import pytest
import numpy as np

try:
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False
    np = None

import radiate as rd
from radiate import (
    IntCodec, FloatCodec, CharCodec, BitCodec, 
    GraphCodec, TreeCodec, PermutationCodec
)


class TestIntCodec:
    """Unit tests for IntCodec."""
    
    @pytest.mark.unit
    def test_int_codec_vector_creation(self):
        """Test creating an integer codec for vectors."""
        codec = IntCodec.vector(length=5, value_range=(0, 10))
        genotype = codec.encode()
        
        assert len(genotype) == 1
        assert len(genotype[0]) == 5
        assert all(0 <= gene.allele() <= 10 for gene in genotype[0])

    @pytest.mark.unit
    def test_int_codec_matrix_creation(self):
        """Test creating an integer codec for matrices."""
        codec = IntCodec.matrix((3, 4), value_range=(0, 100))
        genotype = codec.encode()
        
        assert len(genotype) == 3
        assert all(len(row) == 4 for row in genotype)
        for row in genotype:
            assert all(isinstance(gene.allele(), int) for gene in row)
        assert all(0 <= gene.allele() <= 100 for row in genotype for gene in row)
    
    @pytest.mark.unit
    def test_int_codec_decode(self):
        """Test decoding integer genotypes."""
        codec = IntCodec.vector(length=3, value_range=(0, 5))
        genotype = codec.encode()
        decoded = codec.decode(genotype)
        
        assert len(decoded) == 3
        assert all(isinstance(x, int) for x in decoded)
        assert all(0 <= x <= 5 for x in decoded)
    
    @pytest.mark.unit
    def test_int_codec_bounds(self):
        """Test integer codec respects bounds."""
        codec = IntCodec.vector(length=10, value_range=(-5, 5))
        genotype = codec.encode()
        
        for gene in genotype[0]:

            assert -5 <= gene.allele() <= 5

    @pytest.mark.unit
    def test_int_codec_invalid_bounds(self):
        """Test integer codec handles invalid bounds."""
        with pytest.raises(ValueError):
            IntCodec.vector(length=5, value_range=(10, 5))  # min > max


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

        print(len(genotype), genotype)
        
        assert len(genotype) == 2
        assert all(len(row) == 3 for row in genotype)
        assert all(isinstance(gene.allele(), str) 
                   for row in genotype for gene in row)

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


class TestGraphCodec:
    """Unit tests for GraphCodec."""
    
    @pytest.mark.unit
    def test_graph_codec_directed_creation(self):
        """Test creating a directed graph codec."""
        codec = GraphCodec.directed(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul()],
            edge=rd.Op.weight(),
            output=rd.Op.linear()
        )
        genotype = codec.encode()
        
        assert genotype is not None
        assert len(genotype) > 0
    
    @pytest.mark.unit
    def test_graph_codec_recurrent_creation(self):
        """Test creating a recurrent graph codec."""
        codec = GraphCodec.recurrent(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul()],
            edge=rd.Op.weight(),
            output=rd.Op.linear()
        )
        genotype = codec.encode()
        
        assert genotype is not None
        assert len(genotype) > 0
    
    @pytest.mark.unit
    def test_graph_codec_decode(self):
        """Test decoding graph genotypes."""
        codec = GraphCodec.directed(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul()],
            edge=rd.Op.weight(),
            output=rd.Op.linear()
        )
        genotype = codec.encode()
        print(genotype.py_genotype().gene_type())
        graph = codec.decode(genotype)
        
        assert graph is not None
    
    @pytest.mark.unit
    def test_graph_codec_invalid_shape(self):
        """Test graph codec handles invalid shapes."""
        with pytest.raises(ValueError):
            GraphCodec.directed(
                shape=(0, 1),  # Invalid input size
                vertex=[rd.Op.add()],
                edge=rd.Op.weight(),
                output=rd.Op.linear()
            )


class TestTreeCodec:
    """Unit tests for TreeCodec."""
    
    @pytest.mark.unit
    def test_tree_codec_creation(self):
        """Test creating a tree codec."""
        codec = TreeCodec(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul()],
            leaf=[rd.Op.var(0), rd.Op.var(1)],
            root=rd.Op.linear()
        )
        genotype = codec.encode()
        
        assert genotype is not None
        assert len(genotype) > 0
    
    @pytest.mark.unit
    def test_tree_codec_decode(self):
        """Test decoding tree genotypes."""
        codec = TreeCodec(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul()],
            leaf=[rd.Op.var(0), rd.Op.var(1)],
            root=rd.Op.linear()
        )
        genotype = codec.encode()
        tree = codec.decode(genotype)
        
        assert tree is not None


class TestCodecEdgeCases:
    """Unit tests for codec edge cases and error handling."""
    
    @pytest.mark.unit
    def test_zero_length_codec(self):
        """Test codecs handle zero length gracefully."""
        with pytest.raises(ValueError):
            IntCodec.vector(length=0, value_range=(0, 10))
    
    @pytest.mark.unit
    def test_negative_length_codec(self):
        """Test codecs handle negative length gracefully."""
        with pytest.raises(ValueError):
            FloatCodec.vector(length=-1, value_range=(-1.0, 1.0))
    
    @pytest.mark.unit
    def test_empty_matrix_codec(self):
        """Test matrix codecs handle zero dimensions gracefully."""
        with pytest.raises(ValueError):
            CharCodec.matrix((0, 3))
        with pytest.raises(ValueError):
            BitCodec.matrix((3, 0))
    
    @pytest.mark.unit
    def test_codec_serialization(self):
        """Test that codecs can be serialized/deserialized."""
        codec = IntCodec.vector(length=3, value_range=(0, 10))
        genotype = codec.encode()
        
        # Test that we can decode the same genotype multiple times
        decoded1 = codec.decode(genotype)
        decoded2 = codec.decode(genotype)
        
        assert decoded1 == decoded2 