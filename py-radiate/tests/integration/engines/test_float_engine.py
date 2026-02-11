import radiate as rd
import pytest
import numpy as np


@pytest.mark.integration
def test_engine_float_vector_maximization(random_seed):
    """Test engine with float codec for maximization."""

    # Simple fitness function: maximize sum of squares
    def fitness_func(x: list[float]) -> float:
        return sum(xi**2 for xi in x)

    engine = (
        rd.Engine.float(3, (-1.0, 1.0))
        .fitness(fitness_func)
        .maximizing()
        .size(50)
        .select(rd.Select.boltzmann(4.0), rd.Select.elite())
        .alters(rd.Cross.uniform(0.7), rd.Mutate.arithmetic(0.01))
    )

    result = engine.run([rd.ScoreLimit(2.9), rd.GenerationsLimit(100)])

    # Should find values close to ±1.0
    assert result.score()[0] > 2.5
    assert result.index() <= 100
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == "max"


@pytest.mark.integration
def test_engine_float_matrix_minimization(random_seed):
    """Test engine with float codec for minimization."""

    # Simple fitness function: minimize sum of squares
    def fitness_func(x: np.ndarray) -> float:
        assert isinstance(x, np.ndarray)
        return np.sum(x**2)

    engine = rd.Engine(
        codec=rd.FloatCodec.matrix((2, 2), init_range=(-5.0, 5.0), use_numpy=True),
        fitness_func=fitness_func,
        objective=rd.MIN,
        population_size=50,
        offspring_selector=rd.TournamentSelector(3),
        survivor_selector=rd.EliteSelector(),
        alters=[rd.MeanCrossover(0.7), rd.GaussianMutator(0.1)],
    )

    result = engine.run([rd.ScoreLimit(0.1), rd.GenerationsLimit(200)])

    # Should find values close to 0.0
    assert result.score()[0] < 0.5
    assert result.index() <= 200
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == "min"
