from typing import List
import radiate as rd


def test_engine_can_minimize():
    num_genes = 5
    engine = rd.GeneticEngine(
        codec=rd.IntCodec.vector(num_genes, value_range=(0, 10)),
        fitness_func=lambda x: sum(x),
        objectives="min",
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert result.value() == [0 for _ in range(num_genes)]
    assert result.score() == [0]
    assert result.index() < 500


def test_engine_can_maximize():
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
    assert result.index() < 1000


def test_engine_minimizing_limits():
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


def test_engine_minimizing_graph():
    inputs = [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]
    answers = [[0.0], [1.0], [1.0], [0.0]]

    codec = rd.GraphCodec.directed(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=rd.Regression(inputs, answers),
        objectives="min",
        alters=[
            rd.GraphCrossover(0.5, 0.5),
            rd.OperationMutator(0.07, 0.05),
            rd.GraphMutator(0.1, 0.1),
        ],
    )

    result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)])

    assert result.score()[0] < 0.001
    assert result.index() < 1000


def test_engine_minimizing_tree():
    inputs = [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]
    answers = [[0.0], [1.0], [1.0], [0.0]]

    codec = rd.TreeCodec(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sub()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        root=rd.Op.linear(),
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=rd.Regression(inputs, answers),
        objectives="min",
        alters=[rd.TreeCrossover(0.5), rd.HoistMutator(0.1)],
    )

    result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)])

    assert result.score()[0] < 0.001
    assert result.index() < 1000


def test_engine_raises_on_invalid_alterer():
    engine = rd.GeneticEngine(
        rd.IntCodec.vector(5, (0, 10)),
        lambda x: sum(x),
    )

    try:
        engine.alters([rd.GraphCrossover(0.5, 0.5)])
        assert False, "Expected ValueError for invalid alterer"
    except ValueError as e:
        assert "Alterer GraphCrossover does not support gene type int" in str(e)

