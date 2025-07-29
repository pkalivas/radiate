from typing import List
import radiate as rd
import pytest


class TestEngineErrorHandlingIntegration:
    """Integration tests for error handling and edge cases."""

    @pytest.mark.integration
    def test_engine_empty_population(self):
        """Test engine handles empty population gracefully."""

        def fitness_func(x: List[int]) -> float:
            return sum(x)

        with pytest.raises(ValueError):
            engine = rd.GeneticEngine(
                codec=rd.IntCodec.vector(length=3, value_range=(0, 10)),
                fitness_func=fitness_func,
                objectives="min",
                population_size=0,  # Invalid
            )
            engine.run([rd.GenerationsLimit(10)])

    @pytest.mark.integration
    def test_engine_invalid_limits(self):
        """Test engine handles invalid limits gracefully."""

        def fitness_func(x: List[int]) -> float:
            return sum(x)

        engine = rd.GeneticEngine(
            codec=rd.IntCodec.vector(length=3, value_range=(0, 10)),
            fitness_func=fitness_func,
            objectives="min",
        )

        with pytest.raises(ValueError):
            engine.run([rd.GenerationsLimit(-1)])  # Invalid limit

    def test_engine_raises_on_invalid_alterer(self):
        engine = rd.GeneticEngine(
            rd.IntCodec.vector(5, (0, 10)),
            lambda x: sum(x),
        )

        try:
            engine.alters([rd.GraphCrossover(0.5, 0.5)])
            assert False, "Expected ValueError for invalid alterer"
        except ValueError as e:
            assert "Alterer GraphCrossover does not support gene type int" in str(e)
