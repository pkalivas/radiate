import radiate as rd
import pytest


class TestEngineLimits:
    """Unit tests for engine limits."""

    @pytest.mark.unit
    def test_generations_limit(self, simple_float_engine):
        """Test generations limit functionality."""
        limit = rd.GenerationsLimit(10)
        result = simple_float_engine.run(limit)

        assert result.index() == 10

    @pytest.mark.unit
    def test_time_limit(self, simple_float_engine):
        """Test time limit functionality."""
        limit = rd.SecondsLimit(3)
        result = simple_float_engine.run(limit)

        assert abs(result.duration().total_seconds() - 3) < 0.01

    @pytest.mark.unit
    def test_score_limit(self, simple_float_engine):
        """Test score limit functionality."""
        limit = rd.ScoreLimit(0.01)
        result = simple_float_engine.run(limit)

        assert result.score()[0] <= 0.01

    @pytest.mark.unit
    def test_convergence_limit(self, simple_float_engine):
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
            abs(handler.convergence_data[i] - handler.convergence_data[i - 1])
            < threshold
            for i in range(1, len(handler.convergence_data))
        ), "Convergence limit should ensure scores are within threshold"

    @pytest.mark.unit
    def test_multiple_limits(self, simple_float_engine):
        """Test running with multiple limits."""
        limits = [rd.GenerationsLimit(5), rd.ScoreLimit(0.1), rd.SecondsLimit(2)]
        result = simple_float_engine.run(limits)

        if result.index() < 5 and result.score()[0] > 0.1:
            assert result.duration().total_seconds() < 2, "Should respect time limit"
        elif result.index() == 5:
            assert result.score()[0] > 0.1, "Should respect score limit"
        else:
            assert result.index() == 5, "Should respect generations limit"
