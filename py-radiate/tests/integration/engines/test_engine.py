import radiate as rd
import numpy as np
import pytest


@pytest.mark.integration
def test_engine_maintains_population_size(random_seed):
    import math

    A = 10.0
    RANGE = 5.12
    N_GENES = 2

    def fitness_fn(x: list[float]) -> float:
        value = A * N_GENES
        for i in range(N_GENES):
            value += x[i] ** 2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
        return value

    codec = rd.FloatCodec.vector(N_GENES, init_range=(-RANGE, RANGE))
    population = rd.Population(rd.Phenotype(codec.encode()) for _ in range(107))

    engine = rd.GeneticEngine(
        codec,
        fitness_fn,
        population,
        objective="min",
        alters=[rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.01)],
    )

    result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)])

    assert all(i < 0.001 for i in result.value())
    assert len(result.value()) == N_GENES
    assert result.index() < 1000
    assert len(result.population()) == 107


@pytest.mark.integration
def test_engine_batch_fitness():
    import math

    A = 10.0
    RANGE = 5.12
    N_GENES = 2

    def fitness_fn(x: list[list[float]]) -> list[float]:
        assert len(x) > 1

        results = []
        for member in x:
            value = A * N_GENES
            for i in range(N_GENES):
                value += member[i] ** 2 - A * math.cos(
                    (2.0 * 3.141592653589793 * member[i])
                )
            results.append(value)
        return results

    codec = rd.FloatCodec.vector(N_GENES, init_range=(-RANGE, RANGE))
    population = rd.Population(rd.Phenotype(codec.encode()) for _ in range(107))

    engine = rd.GeneticEngine(
        codec,
        fitness_func=rd.BatchFitness(fitness_fn),
        population=population,
        objective="min",
        alters=[rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.01)],
    )

    result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)])

    assert all(i < 0.001 for i in result.value())
    assert len(result.value()) == N_GENES
    assert result.index() < 1000
    assert len(result.population()) == 107


@pytest.mark.integration
def test_engine_multi_objective(random_seed):
    """Test engine with multi-objective optimization."""

    def fitness_func(x: list[float]) -> list[float]:
        # Two objectives: minimize sum, maximize product
        return [sum(x), np.prod(x)]

    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.vector(length=3, init_range=(-1.0, 1.0)),
        fitness_func=fitness_func,
        objective=["min", "max"],
        population_size=100,
        offspring_selector=rd.TournamentSelector(3),
        survivor_selector=rd.NSGA2Selector(),
        alters=[rd.ArithmeticMutator(0.7), rd.GaussianMutator(0.1)],
    )

    result = engine.run(rd.GenerationsLimit(50))

    assert len(result.score()) == 2, "Should return two objectives"
    assert result.index() == 50, "Should complete within 50 generations"
    assert result.objective() == ["min", "max"]


@pytest.mark.integration
def test_engine_multi_objective_front(simple_multi_objective_engine, random_seed):
    """Test multi-objective engine with Pareto front."""
    result = simple_multi_objective_engine.run(rd.GenerationsLimit(100))

    fitness_values = list(set(map(lambda x: tuple(x.score()), result.front())))

    assert result.objective() == ["min", "min"]
    # Check if the Pareto front is non-dominated
    for i, f1 in enumerate(fitness_values):
        for j, f2 in enumerate(fitness_values):
            if i != j:
                assert not (f1[0] <= f2[0] and f1[1] <= f2[1]), (
                    "Pareto front should be non-dominated"
                )
