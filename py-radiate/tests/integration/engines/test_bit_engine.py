import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_bit_optimization(random_seed):
    """Test engine with bit codec for binary optimization."""
    engine = rd.GeneticEngine(
        codec=rd.BitCodec.vector(10),
        fitness_func=lambda x: sum(1 for bit in x if bit),
        survivor_selector=rd.EliteSelector(),
        alters=[rd.UniformCrossover(0.7), rd.UniformMutator(0.1)],
    )

    result = engine.run([rd.ScoreLimit(10), rd.GenerationsLimit(100)])

    assert result.value() == [True] * 10  # All ones
    assert result.score()[0] == 10.0
    assert result.index() <= 100  # Should converge quickly


@pytest.mark.integration
def test_engine_bit_matrix_optimization(random_seed):
    rows, cols = 4, 4

    def fitness_func(x: list[list[bool]]) -> int:
        assert len(x) == rows
        assert all(len(row) == cols for row in x)
        return sum(1 for row in x for bit in row if bit)

    engine = rd.GeneticEngine(
        codec=rd.BitCodec.matrix((rows, cols)),
        fitness_func=fitness_func,
        survivor_selector=rd.EliteSelector(),
        alters=[rd.UniformCrossover(0.7), rd.UniformMutator(0.1)],
    )

    result = engine.run([rd.ScoreLimit(rows * cols), rd.GenerationsLimit(200)])

    assert result.value() == [[True] * cols for _ in range(rows)]  # All ones
    assert result.score()[0] == rows * cols
    assert result.index() <= 200  # Should converge quickly
