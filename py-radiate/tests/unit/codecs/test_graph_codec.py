import pytest

import radiate as rd
from radiate import (
    GraphCodec,
)


class TestGraphCodec:
    """Unit tests for GraphCodec."""

    @pytest.mark.unit
    def test_graph_codec_directed_creation(self):
        """Test creating a directed graph codec."""
        codec = GraphCodec.directed(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul()],
            edge=rd.Op.weight(),
            output=rd.Op.linear(),
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
            output=rd.Op.linear(),
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
            output=rd.Op.linear(),
        )
        genotype = codec.encode()
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
                output=rd.Op.linear(),
            )
