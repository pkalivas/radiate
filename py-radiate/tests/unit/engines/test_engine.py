from typing import List
import radiate as rd
import numpy as np
import pytest


class TestEngineBasicIntegration:
    """Basic integration tests for GeneticEngine functionality."""

    @pytest.mark.integration
    def test_engine_int_minimization(self, random_seed):
        num_genes = 5
        engine = rd.GeneticEngine(
            codec=rd.IntCodec.vector(num_genes, value_range=(0, 10)),
            fitness_func=lambda x: sum(x),
            objectives="min",
        )

        result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

        assert result.value() == [0 for _ in range(num_genes)]
        assert result.score() == [0]
        assert result.index() <= 500

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
        assert result.index() <= 100

    def test_engine_can_maximize(self):
        target = "Testing, Radiate!"

        def fitness_func(x: List[str]) -> int:
            return sum(1 for i in range(len(target)) if x[i] == target[i])

        engine = rd.GeneticEngine(
            codec=rd.CharCodec.vector(len(target)),
            fitness_func=fitness_func,
            offspring_selector=rd.BoltzmannSelector(4),
        )

        result = engine.run([rd.ScoreLimit(len(target)), rd.GenerationsLimit(1000)])

        assert result.value() == list(target)
        assert result.score() == [len(target)]
        assert result.index() <= 1000

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
        assert result.index() <= 100

    @pytest.mark.integration
    def test_engine_minimizing_limits(self):
        import math

        A = 10.0
        RANGE = 5.12
        N_GENES = 2

        def fitness_fn(x: List[float]) -> float:
            value = A * N_GENES
            for i in range(N_GENES):
                value += x[i] ** 2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
            return value

        engine = rd.GeneticEngine(rd.FloatCodec.vector(2, (-RANGE, RANGE)), fitness_fn)
        engine.minimizing()
        engine.alters([rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.01)])

        result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)])

        assert all(i < 0.001 for i in result.value())
        assert len(result.value()) == N_GENES
        assert result.index() < 1000

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
        assert result.index() <= 500

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
        assert result.index() <= 300

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

        assert result.index() <= 100
        assert len(set(result.value())) == 5
        assert all(0 <= x < 5 for x in result.value())

    @pytest.mark.integration
    def test_engine_multi_objective(self, random_seed):
        """Test engine with multi-objective optimization."""

        def fitness_func(x: List[float]) -> List[float]:
            # Two objectives: minimize sum, maximize product
            return [sum(x), np.prod(x)]

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=3, value_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objectives=["min", "max"],
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.NSGA2Selector(),
            alters=[rd.ArithmeticMutator(0.7), rd.GaussianMutator(0.1)],
        )

        result = engine.run(rd.GenerationsLimit(50))

        assert len(result.score()) == 2, "Should return two objectives"
        assert result.index() == 50, "Should complete within 50 generations"
