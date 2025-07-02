"""
Integration tests for Radiate genetic engine.

These tests verify that different components work together correctly.
"""

import pytest
import numpy as np
from typing import List

import radiate as rd


class TestEngineBasicIntegration:
    """Integration tests for basic engine functionality."""

    @pytest.mark.integration
    def test_engine_int_minimization(self, random_seed):
        """Test engine with integer codec for minimization."""

        # Simple fitness function: minimize sum
        def fitness_func(x: List[int]) -> float:
            return sum(x)

        engine = rd.GeneticEngine(
            codec=rd.IntCodec.vector(length=5, value_range=(0, 10)),
            fitness_func=fitness_func,
            objectives="min",
            population_size=50,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.UniformCrossover(0.7), rd.ArithmeticMutator(0.1)],
        )

        result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(100)])

        assert result.value() == [0] * 5  # All zeros
        assert result.score()[0] == 0.0
        assert result.index() < 100

    @pytest.mark.integration
    def test_engine_float_maximization(self, random_seed):
        """Test engine with float codec for maximization."""

        # Simple fitness function: maximize sum of squares
        def fitness_func(x: List[float]) -> float:
            return sum(xi**2 for xi in x)

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=3, value_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objectives="max",
            population_size=50,
            offspring_selector=rd.BoltzmannSelector(4.0),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.MeanCrossover(0.7), rd.GaussianMutator(0.1)],
        )

        result = engine.run([rd.ScoreLimit(2.9), rd.GenerationsLimit(100)])

        # Should find values close to ±1.0
        assert result.score()[0] > 2.5
        assert result.index() < 100

    @pytest.mark.integration
    def test_engine_char_string_matching(self, random_seed):
        """Test engine with character codec for string matching."""
        target = "HELLO"

        def fitness_func(x: List[str]) -> float:
            return sum(1 for i, c in enumerate(x) if c == target[i])

        engine = rd.GeneticEngine(
            codec=rd.CharCodec.vector(length=len(target)),
            fitness_func=fitness_func,
            objectives="max",
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.UniformCrossover(0.8), rd.UniformMutator(0.1)],
        )

        result = engine.run([rd.ScoreLimit(len(target)), rd.GenerationsLimit(200)])

        assert result.value() == list(target)
        assert result.score()[0] == len(target)
        assert result.index() < 200

    @pytest.mark.integration
    def test_engine_bit_optimization(self, random_seed):
        """Test engine with bit codec for binary optimization."""

        # Maximize number of 1s
        def fitness_func(x: List[bool]) -> float:
            return sum(1 for bit in x if bit)

        engine = rd.GeneticEngine(
            codec=rd.BitCodec.vector(length=10),
            fitness_func=fitness_func,
            objectives="max",
            population_size=50,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.UniformCrossover(0.7), rd.UniformMutator(0.1)],
        )

        result = engine.run([rd.ScoreLimit(10), rd.GenerationsLimit(100)])

        assert result.value() == [True] * 10  # All ones
        assert result.score()[0] == 10.0
        assert result.index() < 100


class TestEngineAdvancedIntegration:
    """Integration tests for advanced engine features."""

    @pytest.mark.integration
    @pytest.mark.slow
    def test_engine_graph_xor(self, xor_dataset, random_seed):
        """Test engine with graph codec for XOR problem."""
        inputs, outputs = xor_dataset

        engine = rd.GeneticEngine(
            codec=rd.GraphCodec.directed(
                shape=(2, 1),
                vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
                edge=rd.Op.weight(),
                output=rd.Op.linear(),
            ),
            fitness_func=rd.Regression(inputs, outputs),
            objectives="min",
            population_size=100,
            offspring_selector=rd.BoltzmannSelector(4.0),
            survivor_selector=rd.EliteSelector(),
            alters=[
                rd.GraphCrossover(0.5, 0.5),
                rd.OperationMutator(0.07, 0.05),
                rd.GraphMutator(0.1, 0.1),
            ],
        )

        result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(500)])

        assert result.score()[0] < 0.001
        assert result.index() < 500

    @pytest.mark.integration
    @pytest.mark.slow
    def test_engine_tree_regression(self, simple_regression_dataset, random_seed):
        """Test engine with tree codec for regression."""
        inputs, outputs = simple_regression_dataset

        engine = rd.GeneticEngine(
            codec=rd.TreeCodec(
                shape=(1, 1),
                vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sub()],
                leaf=[rd.Op.var(0)],
                root=rd.Op.linear(),
            ),
            fitness_func=rd.Regression(inputs, outputs),
            objectives="min",
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.TreeCrossover(0.5), rd.HoistMutator(0.1)],
        )

        result = engine.run([rd.ScoreLimit(0.1), rd.GenerationsLimit(300)])

        assert result.score()[0] < 0.1
        assert result.index() < 300

    @pytest.mark.integration
    def test_engine_permutation_tsp(self, random_seed):
        """Test engine with permutation codec for TSP-like problem."""

        # Simple TSP-like fitness: minimize sum of adjacent differences
        def fitness_func(x: List[int]) -> float:
            return sum(abs(x[i] - x[i - 1]) for i in range(1, len(x)))

        engine = rd.GeneticEngine(
            codec=rd.PermutationCodec([0, 1, 2, 3, 4]),
            fitness_func=fitness_func,
            objectives="min",
            population_size=50,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.PartiallyMappedCrossover(0.7), rd.InversionMutator(0.1)],
        )

        result = engine.run([rd.GenerationsLimit(100)])

        assert result.index() == 100
        # Check that result is a valid permutation
        assert len(set(result.value())) == 5
        assert all(0 <= x < 5 for x in result.value())


class TestEngineMultiObjectiveIntegration:
    """Integration tests for multi-objective optimization."""

    @pytest.mark.integration
    def test_engine_multi_objective(self, random_seed):
        """Test engine with multi-objective optimization."""

        def fitness_func(x: List[float]) -> List[float]:
            # Two objectives: minimize sum, maximize product
            return [sum(x), -np.prod(x)]  # Negative for maximization

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=3, value_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objectives=["min", "max"],
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.NSGA2Selector(),
            alters=[rd.ArithmeticMutator(0.7), rd.GaussianMutator(0.1)],
        )

        result = engine.run([rd.GenerationsLimit(50)])

        assert len(result.score()) == 2, "Should return two objectives"
        assert result.index() <= 50, "Should complete within 50 generations"


class TestEngineErrorHandlingIntegration:
    """Integration tests for error handling and edge cases."""

    # @pytest.mark.integration
    # def test_engine_invalid_fitness_function(self):
    #     """Test engine handles invalid fitness function gracefully."""
    #     def invalid_fitness(x):
    #         raise ValueError("Test error")

    #     engine = rd.GeneticEngine(
    #         codec=rd.IntCodec.vector(length=3, value_range=(0, 10)),
    #         fitness_func=invalid_fitness,
    #         objectives="min"
    #     )

    #     # Should handle the error gracefully
    #     result = engine.run([rd.GenerationsLimit(10)])

    #     assert result.index() < 10
    #     # Should have some score (even if poor)
    #     assert result.score() is not None

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


class TestEnginePerformanceIntegration:
    """Integration tests for performance characteristics."""

    @pytest.mark.integration
    @pytest.mark.performance
    def test_engine_large_population_performance(
        self, performance_benchmark, random_seed
    ):
        """Test engine performance with large population."""

        def fitness_func(x: List[float]) -> float:
            return sum(xi**2 for xi in x)

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=10, value_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objectives="min",
            population_size=1000,  # Large population
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.ArithmeticMutator(0.7), rd.IntermediateCrossover(0.1)],
        )

        result, execution_time = performance_benchmark.time_function(
            engine.run, [rd.GenerationsLimit(10)]
        )

        assert result.index() == 10
        assert execution_time < 30.0  # Should complete within 30 seconds

    @pytest.mark.integration
    @pytest.mark.performance
    def test_engine_memory_usage(self, performance_benchmark, random_seed):
        """Test engine memory usage."""

        def fitness_func(x: List[float]) -> float:
            return sum(xi**2 for xi in x)

        initial_memory = performance_benchmark.memory_usage()

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=50, value_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objectives="min",
            population_size=500,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.ArithmeticMutator(0.7), rd.IntermediateCrossover(0.1)],
        )

        engine.run([rd.GenerationsLimit(20)])

        final_memory = performance_benchmark.memory_usage()

        if initial_memory and final_memory:
            memory_increase = final_memory - initial_memory
            assert memory_increase < 100.0  # Should not use more than 100MB extra


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
