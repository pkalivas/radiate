import pytest

import radiate as rd
from radiate import TreeCodec


class TestTreeCodec:
    """Unit tests for TreeCodec."""

    @pytest.mark.unit
    def test_tree_codec_creation(self):
        """Test creating a tree codec."""
        codec = TreeCodec(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul()],
            leaf=[rd.Op.var(0), rd.Op.var(1)],
            root=rd.Op.linear(),
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
            root=rd.Op.linear(),
        )
        genotype = codec.encode()
        tree = codec.decode(genotype)

        assert tree is not None
