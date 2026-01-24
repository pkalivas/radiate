import pytest
import radiate as rd


@pytest.mark.unit
def test_fixed_rate():
    """Test FixedRate input."""
    rate_input = rd.Rate.fixed(0.5)
    for i in range(10):
        assert rate_input.value(i) == 0.5


@pytest.mark.unit
def test_linear_rate():
    """Test LinearRate input."""
    rate_input = rd.Rate.linear(0.0, 1.0, 10)
    expected_values = [i / 10.0 for i in range(11)]
    for i, expected in enumerate(expected_values):
        assert pytest.approx(rate_input.value(i), 0.01) == expected


@pytest.mark.unit
def test_step_rate():
    """Test StepRate input."""
    steps = [(0, 0.0), (5, 1.0), (10, 0.5)]
    rate_input = rd.Rate.stepwise(steps=steps)
    expected_values = [0.0] * 5 + [1.0] * 5 + [0.5] * 5
    for i, expected in enumerate(expected_values):
        assert pytest.approx(rate_input.value(i), 0.01) == expected
