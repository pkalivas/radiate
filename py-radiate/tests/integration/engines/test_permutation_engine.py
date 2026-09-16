import pytest

import radiate as rd


@pytest.mark.integration
def test_engine_permutation_tsp(random_seed):
    """Test engine with permutation codec for TSP-like problem."""

    # Simple TSP-like fitness: minimize sum of adjacent differences
    def fit(x: list[int]) -> float:
        return sum(abs(x[i] - x[i - 1]) for i in range(1, len(x)))

    result = (
        rd.Engine.permutation([0, 1, 2, 3, 4])
        .fitness(fit)
        .minimizing()
        .size(50)
        .select(rd.Select.tournament(k=3), rd.Select.elite())
        .alter(rd.Cross.pmx(rate=0.7), rd.Mutate.inversion(rate=0.1))
        .limit(rd.Limit.score(5), rd.Limit.generations(100))
    ).run()

    assert result.index() <= 100
    assert len(set(result.value())) == 5
    assert all(0 <= x < 5 for x in result.value())
