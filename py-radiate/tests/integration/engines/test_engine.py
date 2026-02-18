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

    engine = (
        rd.Engine.float(N_GENES, init_range=(-RANGE, RANGE))
        .fitness(fitness_fn)
        .minimizing()
        .population(population)
        .alters(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.01))
    )

    result = engine.run(rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000))

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

    @rd.fitness(batch=True)
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

    engine = (
        rd.Engine.float(N_GENES, init_range=(-RANGE, RANGE))
        .fitness(fitness_fn)
        .minimizing()
        .population(population)
        .alters(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.01))
    )

    result = engine.run(rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000))

    assert all(i < 0.001 for i in result.value())
    assert len(result.value()) == N_GENES
    assert result.index() < 1000
    assert len(result.population()) == 107


@pytest.mark.integration
def test_engine_multi_objective(random_seed):
    """Test engine with multi-objective optimization."""

    def fitness_func(x: list[float]) -> list[float]:
        # Two objectives: minimize sum, maximize product
        return [sum(x), np.prod(x)]  # type: ignore

    engine = (
        rd.Engine.float(3, init_range=(-10.0, 10.0))
        .fitness(fitness_func)
        .objective(rd.MIN, rd.MAX)
        .alters(rd.Cross.uniform(0.7), rd.Mutate.arithmetic(0.1))
        .select(rd.Select.tournament(3), rd.Select.nsga2())
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

    assert result.objective() == [rd.MIN, rd.MIN], (
        "Should be minimizing both objectives"
    )
    # Check if the Pareto front is non-dominated
    for i, f1 in enumerate(fitness_values):
        for j, f2 in enumerate(fitness_values):
            if i != j:
                assert not (f1[0] <= f2[0] and f1[1] <= f2[1]), (
                    "Pareto front should be non-dominated"
                )
