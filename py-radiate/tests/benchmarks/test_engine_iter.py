import radiate as rd
import numpy as np
import pytest


def sphere(x: list[float]) -> float:
    return -float(np.dot(x, x))


def build_engine(pop: int, dim: int, threads: int):
    codec = rd.FloatCodec.vector(length=dim, init_range=(-5.0, 5.0), use_numpy=True)

    return rd.GeneticEngine(
        codec=codec,
        fitness_func=sphere,
        population_size=pop,
        offspring_fraction=0.8,
        objectives="max",
        alters=[rd.UniformMutator(0.05), rd.UniformCrossover(0.9)],
        survivor_selector=rd.TournamentSelector(k=3),
        offspring_selector=rd.RouletteSelector(),
    )


def bench_once(pop: int, dim: int, threads: int):
    eng = build_engine(pop, dim, threads)
    _ = next(eng)


@pytest.mark.bench
def test_bench_graph_regression(benchmark, simple_regression_dataset, random_seed):
    inputs, outputs = simple_regression_dataset

    codec = rd.GraphCodec.directed(
        shape=(1, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=rd.Regression(inputs, outputs),
        objectives="min",
        population_size=100,
        species_threshold=0.1,
        alters=[
            rd.GraphCrossover(0.5, 0.5),
            rd.OperationMutator(0.07, 0.05),
            rd.GraphMutator(0.1, 0.1),
        ],
    )

    result = benchmark(
        lambda: engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(500)])
    )
    assert result is not None


@pytest.mark.bench
def test_engine_float_maximization_benchmark(benchmark, random_seed):
    """Test engine with float codec for maximization."""

    # Simple fitness function: maximize sum of squares
    def fitness_func(x: list[float]) -> float:
        return sum(xi**2 for xi in x)

    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.vector(length=30, init_range=(-1.0, 1.0)),
        fitness_func=fitness_func,
        objectives="max",
        population_size=100,
        offspring_selector=rd.BoltzmannSelector(4.0),
        survivor_selector=rd.EliteSelector(),
        alters=[rd.MeanCrossover(0.7), rd.GaussianMutator(0.1)],
    )

    result = benchmark(
        lambda: engine.run([rd.ScoreLimit(30), rd.GenerationsLimit(100)])
    )
    assert result is not None


@pytest.mark.bench
def test_engine_benchmark_next_small(benchmark, random_seed):
    result = benchmark(lambda: bench_once(pop=512, dim=16, threads=0))
    assert result is None


@pytest.mark.bench
def test_python_callable_fitness_overhead_benchmark(benchmark, random_seed):
    eng = build_engine(pop=2048, dim=16, threads=0)
    benchmark(lambda: next(eng))
