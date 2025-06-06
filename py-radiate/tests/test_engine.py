from __future__ import annotations

import radiate as rd


def test_simple_min_sum_engine():
    num_genes = 5
    engine = rd.GeneticEngine(
        codec=rd.IntCodec.vector(num_genes, value_range=(0, 10)),
        fitness_func=lambda x: sum(x),
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert result.value() == [0 for _ in range(num_genes)]
    assert result.score() == [0]
    assert result.index() < 500


def test_simple_string_matching_engine():
    target = "Hello, Radiate!"

    def fitness_func(x):
        return sum(1 for i in range(len(target)) if x[i] == target[i])

    engine = rd.GeneticEngine(
        codec=rd.CharCodec.vector(len(target)),
        fitness_func=fitness_func,
        objectives="max",
        offspring_selector=rd.BoltzmannSelector(4),
    )

    result = engine.run([rd.ScoreLimit(len(target)), rd.GenerationsLimit(1000)])

    assert result.value() == list(target)
    assert result.score() == [len(target)]
    assert result.index() < 1000


def test_rastrigin_function_engine():
    import math

    A = 10.0
    RANGE = 5.12
    N_GENES = 2

    def fitness_fn(x):
        value = A * N_GENES
        for i in range(N_GENES):
            value += x[i] ** 2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
        return value

    engine = rd.GeneticEngine(rd.FloatCodec.vector(2, (-RANGE, RANGE)), fitness_fn)
    engine.alters([rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.01)])

    result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)])

    assert all(i < 0.001 for i in result.value())
    assert len(result.value()) == N_GENES
    assert result.index() < 1000
