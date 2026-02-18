import pytest
import radiate as rd


@pytest.mark.unit
def test_gp_op_creation():
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

    assert add_op.eval(1, 2) == 3
    assert sub_op.eval(5, 3) == 2
    assert mul_op.eval(4, 6) == 24
    assert div_op.eval(10, 2) == 5

    assert rd.Op.sum().eval(1, 2, 3) == 6
    assert rd.Op.prod().eval(2, 3, 4) == 24

    assert rd.Op.var(0).eval(5) == 5
    assert rd.Op.var(3).eval(1, 2, 3, 4) == 4
