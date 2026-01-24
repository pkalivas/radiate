import radiate as rd
import numpy as np
import pytest


@pytest.mark.integration
def test_engine_int_minimization(random_seed):
    num_genes = 5
    engine = rd.GeneticEngine(
        codec=rd.IntCodec.vector(num_genes, init_range=(0, 10)),
        fitness_func=lambda x: sum(x),
        objective="min",
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert result.value() == [0 for _ in range(num_genes)]
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == "min"


@pytest.mark.integration
def test_engine_int_vector_nparray(random_seed):
    num_genes = 5

    def fitness_func(x: np.ndarray) -> float:
        assert isinstance(x, np.ndarray)
        assert np.all(x >= -10) and np.all(x <= 50)
        return np.sum(x)

    codec = rd.IntCodec.vector(
        num_genes, init_range=(0, 10), bounds=(-10, 50), use_numpy=True
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_func,
        objective="min",
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert np.array_equal(result.value(), np.array([0 for _ in range(num_genes)]))
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == "min"


@pytest.mark.integration
def test_engine_int_matrix_nparray(random_seed):
    rows, cols = 3, 4

    def fitness_func(x: np.ndarray) -> float:
        assert isinstance(x, np.ndarray)
        assert x.shape == (rows, cols)
        assert np.all(x >= -5) and np.all(x <= 20)
        return float(np.sum(x))

    codec = rd.IntCodec.matrix(
        shape=(rows, cols), init_range=(0, 10), bounds=(-5, 20), use_numpy=True
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_func,
        objective="min",
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert np.array_equal(result.value(), np.zeros((rows, cols), dtype=int))
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == "min"


@pytest.mark.integration
def test_engine_int_jagged_matrix(random_seed):
    shape = [2, 3, 4]

    def fitness_func(x: list[list[int]]) -> float:
        assert isinstance(x, list)
        assert all(isinstance(row, list) for row in x)
        assert len(x) == 3
        for i, row in enumerate(x):
            assert len(row) == shape[i]
            for gene in row:
                assert isinstance(gene, int)
                assert -5 <= gene <= 20
        return float(sum(sum(row) for row in x))

    # Create a jagged matrix codec - right now (1/23/26) this doesn't support non-square shapes
    codec = rd.IntCodec.matrix(shape=shape, init_range=(0, 10), bounds=(-5, 20))

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_func,
        objective="min",
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert result.value() == [[0 for _ in range(n)] for n in shape]
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == "min"
