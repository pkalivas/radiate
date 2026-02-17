import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_bit_optimization(random_seed):
    """Test engine with bit codec for binary optimization."""
    engine = (
        rd.Engine.bit(10)
        .fitness(lambda x: sum(1 for bit in x if bit))  # Maximize number of ones
        .select(rd.Select.elite())
        .alters(rd.Cross.uniform(0.7), rd.Mutate.uniform(0.1))
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

    engine = (
        rd.Engine.bit((rows, cols))
        .fitness(fitness_func)
        .select(rd.Select.elite())
        .alters(rd.Cross.uniform(0.7), rd.Mutate.uniform(0.1))
    )

    result = engine.run([rd.ScoreLimit(rows * cols), rd.GenerationsLimit(200)])

    assert result.value() == [[True] * cols for _ in range(rows)]  # All ones
    assert result.score()[0] == rows * cols
    assert result.index() <= 200  # Should converge quickly


@pytest.mark.integration
def test_engine_bit_can_maximize(simple_bit_20_bit_engine, random_seed):
    """Test engine with bit codec for maximizing number of ones."""
    engine = simple_bit_20_bit_engine.fitness(
        lambda x: sum(1 for bit in x if bit)
    ).maximizing()

    result = engine.run([rd.ScoreLimit(20), rd.GenerationsLimit(100)])

    assert result.value() == [True] * 20  # All ones
    assert result.score()[0] == 20.0
    assert result.index() <= 100  # Should converge quickly


@pytest.mark.integration
def test_engine_bit_can_co_evolve_two_chromosomes(random_seed):
    """Test engine with bit codec for co-evolving two chromosomes."""

    def fitness_func(x: list[list[bool]]) -> int:
        assert len(x) == 2
        sum_one = sum(1 for bit in x[0] if bit)
        sum_two = sum(1 for bit in x[1] if not bit)
        return sum_one + sum_two

    # Two chromosomes both with 20 genes in it.
    # We want the first chromosome to be all ones and the second chromosome to be all zeros.
    engine = (
        rd.Engine.bit(shape=[20, 20])
        .fitness(fitness_func)
        .minimizing()
        .alters(rd.Cross.uniform(0.5), rd.Mutate.uniform(0.1))
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(200)])

    assert result.value()[0] == [False] * 20  # First chromosome all zeros
    assert result.value()[1] == [True] * 20  # Second chromosome all ones
    assert result.score()[0] == 0.0
    assert result.index() <= 200  # Should converge quickly
