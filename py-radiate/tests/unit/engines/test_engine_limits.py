import radiate as rd
import pytest


@pytest.mark.unit
def test_generations_limit(simple_float_engine):
    """Test generations limit functionality."""
    limit = rd.Limit.generations(10)
    result = simple_float_engine.run(limit)

    assert result.index() == 10


@pytest.mark.unit
def test_time_limit(simple_float_engine):
    """Test time limit functionality."""
    limit = rd.Limit.seconds(3)
    result = simple_float_engine.run(limit)

    assert abs(result.duration().total_seconds() - 3) < 0.01


@pytest.mark.unit
def test_score_limit(simple_float_engine):
    """Test score limit functionality."""
    limit = rd.Limit.score(0.01)
    result = simple_float_engine.run(limit)

    assert result.score()[0] <= 0.01


@pytest.mark.unit
def test_convergence_limit(simple_float_engine, random_seed):
    """Test convergence limit functionality."""

    window_size = 15
    threshold = 0.0001

    class Subscriber(rd.EventHandler):
        def __init__(self):
            super().__init__(rd.EventType.EPOCH_COMPLETE)
            self.convergence_data = []

        def on_event(self, generation):
            if len(self.convergence_data) >= window_size:
                self.convergence_data.pop(0)
            self.convergence_data.append(generation.score())

    handler = Subscriber()
    simple_float_engine.subscribe(handler)

    simple_float_engine.run(rd.ConvergenceLimit(window_size, threshold))

    assert len(handler.convergence_data) == window_size
    assert all(
        abs(handler.convergence_data[i] - handler.convergence_data[i - 1]) < threshold
        for i in range(1, len(handler.convergence_data))
    ), "Convergence limit should ensure scores are within threshold"


@pytest.mark.unit
def test_expr_limit(simple_float_engine):
    """Test expression-based limit."""
    limit = rd.Expr.select("index") >= 10

    result = simple_float_engine.run(rd.Limit.expr(limit))

    assert result.index() == 10, "Expression limit should stop at index >= 10"
