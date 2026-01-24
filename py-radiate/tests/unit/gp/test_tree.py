import pytest
import radiate as rd


@pytest.mark.unit
def test_gp_tree_creation():
    """Test GP Tree creation."""
    codec = rd.TreeCodec(
        vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        max_size=30,
    )

    tree = codec.decode(codec.encode())

    assert tree is not None
    assert tree.eval([[1.0, 2.0, 3.0]]) is not None


@pytest.mark.unit
def test_gp_tree_eval_with_single_input():
    """Test GP Tree evaluation with single input list."""
    codec = rd.TreeCodec(
        vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        max_size=30,
    )

    tree = codec.decode(codec.encode())

    result = tree.eval([[1.0, 2.0]])
    assert isinstance(result, list)
    assert len(result) > 0


@pytest.mark.unit
def test_gp_tree_eval_with_multiple_inputs():
    """Test GP Tree evaluation with multiple input lists."""
    codec = rd.TreeCodec(
        vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        max_size=30,
    )

    tree = codec.decode(codec.encode())
    inputs = [
        [1.0, 2.0],
        [3.0, 4.0],
        [5.0, 6.0],
    ]
    result = tree.eval(inputs)
    assert isinstance(result, list)
    assert len(result) == len(inputs)


@pytest.mark.unit
def test_gp_tree_eval_with_invalid_input():
    """Test GP Tree evaluation with invalid input."""
    # Create a simple tree
    codec = rd.TreeCodec(
        vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        max_size=30,
    )

    tree = codec.decode(codec.encode())

    with pytest.raises(TypeError):
        tree.eval("invalid")

    with pytest.raises(TypeError):
        tree.eval([[1.0, "invalid", 3.0]])
