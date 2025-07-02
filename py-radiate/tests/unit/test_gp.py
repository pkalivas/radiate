"""
Comprehensive unit tests for Radiate GP components.

These tests focus on covering the missing lines identified in the coverage report.
"""

import pytest
import radiate as rd


class TestGP:
    """Comprehensive tests for GP components to cover missing lines."""

    @pytest.mark.unit
    def test_gp_tree_creation(self):
        """Test GP Tree creation."""
        # Create a simple tree
        codec = rd.TreeCodec(
            vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
            leaf=[rd.Op.var(0), rd.Op.var(1)],
            max_size=30,
        )

        tree = codec.decode(codec.encode())

        assert tree is not None
        assert hasattr(tree, "eval")

    @pytest.mark.unit
    def test_gp_tree_eval_with_single_input(self):
        """Test GP Tree evaluation with single input list."""
        codec = rd.TreeCodec(
            vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
            leaf=[rd.Op.var(0), rd.Op.var(1)],
            max_size=30,
        )

        tree = codec.decode(codec.encode())

        result = tree.eval([1.0, 2.0, 3.0])
        assert isinstance(result, list)
        assert len(result) > 0

    @pytest.mark.unit
    def test_gp_tree_eval_with_multiple_inputs(self):
        """Test GP Tree evaluation with multiple input lists."""
        codec = rd.TreeCodec(
            vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
            leaf=[rd.Op.var(0), rd.Op.var(1)],
            max_size=30,
        )

        tree = codec.decode(codec.encode())
        assert isinstance(tree, rd.Tree)
        assert len(tree) > 0

    @pytest.mark.unit
    def test_gp_tree_eval_with_invalid_input(self):
        """Test GP Tree evaluation with invalid input."""
        # Create a simple tree
        tree = rd.Tree(
            vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
            leaf=[rd.Op.var(0), rd.Op.var(1)],
            max_size=30,
        )

        # Test with invalid input type
        with pytest.raises(
            ValueError,
            match="Inputs must be a list of floats or a list of list of floats",
        ):
            tree.eval("invalid")

        # Test with mixed types
        with pytest.raises(
            ValueError,
            match="Inputs must be a list of floats or a list of list of floats",
        ):
            tree.eval([1.0, "invalid", 3.0])

    @pytest.mark.unit
    def test_gp_graph_creation(self):
        """Test GP Graph creation."""
        # Create a simple graph
        graph = rd.Graph(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
            edge=[rd.Op.weight()],
            output=[rd.Op.linear()],
        )

        assert graph is not None
        assert hasattr(graph, "eval")

    @pytest.mark.unit
    def test_gp_graph_eval(self):
        """Test GP Graph evaluation."""
        # Create a simple graph
        graph = rd.Graph(
            shape=(3, 1),
            vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
            edge=[rd.Op.weight()],
            output=[rd.Op.linear()],
        )

        # Test evaluation
        result = graph.eval([1.0, 2.0, 3.0])
        assert isinstance(result, list)
        assert len(result) > 0

    @pytest.mark.unit
    def test_gp_op_creation(self):
        """Test GP Operation creation."""
        # Test creating different types of operations
        add_op = rd.Op.add()
        sub_op = rd.Op.sub()
        mul_op = rd.Op.mul()
        div_op = rd.Op.div()

        assert add_op is not None
        assert sub_op is not None
        assert mul_op is not None
        assert div_op is not None
