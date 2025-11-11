import radiate as rd
import pytest


class TestEngineErrorHandlingIntegration:
    """Integration tests for error handling and edge cases."""

    @pytest.mark.integration
    def test_engine_empty_population(self):
        """Test engine handles empty population gracefully."""

        def fitness_func(x: list[int]) -> float:
            return sum(x)

        with pytest.raises(ValueError):
            engine = rd.GeneticEngine(
                codec=rd.IntCodec.vector(length=3, init_range=(0, 10)),
                fitness_func=fitness_func,
                objective="min",
                population_size=0,  # Invalid
            )
            engine.run([rd.GenerationsLimit(10)])

    @pytest.mark.integration
    def test_engine_invalid_limits(self):
        """Test engine handles invalid limits gracefully."""

        def fitness_func(x: list[int]) -> float:
            return sum(x)

        engine = rd.GeneticEngine(
            codec=rd.IntCodec.vector(length=3, init_range=(0, 10)),
            fitness_func=fitness_func,
            objective="min",
        )

        with pytest.raises(ValueError):
            engine.run([rd.GenerationsLimit(-1)])  # Invalid limit
