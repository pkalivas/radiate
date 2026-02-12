import pytest
import radiate as rd


@pytest.mark.unit
def test_dtype_max_min():
    """Test that the max and min values for UInt8 are correct."""
    assert rd.UInt8.max() == 255
    assert rd.UInt8.min() == 0

    assert rd.UInt16.max() == 65535
    assert rd.UInt16.min() == 0

    assert rd.UInt32.max() == 4294967295
    assert rd.UInt32.min() == 0

    assert rd.UInt64.max() == 18446744073709551615
    assert rd.UInt64.min() == 0

    assert rd.UInt128.max() == 340282366920938463463374607431768211455
    assert rd.UInt128.min() == 0

    assert rd.Int8.max() == 127
    assert rd.Int8.min() == -128

    assert rd.Int16.max() == 32767
    assert rd.Int16.min() == -32768

    assert rd.Int32.max() == 2147483647
    assert rd.Int32.min() == -2147483648

    assert rd.Int64.max() == 9223372036854775807
    assert rd.Int64.min() == -9223372036854775808

    assert rd.Int128.max() == 170141183460469231731687303715884105727
    assert rd.Int128.min() == -170141183460469231731687303715884105728

    assert rd.Float32.max() == 3.4028234663852886e38
    assert rd.Float32.min() == -3.4028234663852886e38

    assert rd.Float64.max() == 1.7976931348623157e308
    assert rd.Float64.min() == -1.7976931348623157e308
