import pytest
import radiate as rd


@pytest.mark.unit
def test_when_then_expr():
    """Test that we can create a when-then expression and that it has the correct structure."""
    # expr = rd.metric("scores").mean() <= 0.01
    # when_then_expr = (
    #     rd.when(expr).then(rd.metric("scores").min() <= 0.01).otherwise(0.5)
    # )
    # print(when_then_expr)

    metrics = rd.MetricSet(
        {
            "one": list(range(10)),
            "two": list(range(10, 20)),
        }
    )

    print(len(metrics))
