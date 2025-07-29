from typing import List
import radiate as rd
import pytest


class TestEngineConvergenceIntegration:
    """Integration tests for convergence behavior."""

    @pytest.mark.integration
    def test_engine_convergence_curve(self, random_seed):
        """Test engine convergence over generations."""

        def fitness_func(x: List[float]) -> float:
            return sum(xi**2 for xi in x)

        class Subscriber(rd.EventHandler):
            def __init__(self):
                super().__init__(rd.EventType.EPOCH_COMPLETE)
                self.convergence_data = []

            def on_event(self, generation):
                self.convergence_data.append(generation["score"])

        handler = Subscriber()

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=5, value_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objectives="min",
            population_size=50,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            subscribe=handler,
            alters=[rd.ArithmeticMutator(0.7), rd.IntermediateCrossover(0.1)],
        )

        result = engine.run([rd.GenerationsLimit(20)])

        # Check convergence behavior
        assert len(handler.convergence_data) == 20
        assert (
            handler.convergence_data[-1] <= handler.convergence_data[0]
        )  # Should improve or stay same
        assert result.score()[0] == handler.convergence_data[-1]
