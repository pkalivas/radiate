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
