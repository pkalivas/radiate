import pytest

import radiate as rd


@pytest.mark.integration
def test_engine_char_vector():
    target = "Testing, Radiate!"

    def fitness_func(x: list[str]) -> int:
        return sum(1 for i in range(len(target)) if x[i] == target[i])

    engine = (
        rd.Engine.char(len(target))
        .fitness(fitness_func)
        .select(rd.Select.boltzmann(4))
        .alters(rd.Cross.uniform(0.5), rd.Mutate.uniform(0.1))
        .limit(rd.Limit.score(len(target)), rd.Limit.generations(1000))
    )

    result = engine.run()

    assert result.value() == list(target)
    assert result.score() == [len(target)]
    assert result.index() <= 1000


@pytest.mark.integration
def test_engine_char_matrix(random_seed):
    shape = [5, 7, 5]
    target = ["Hello", "Radiate", "World"]

    def fit(x: list[list[str]]) -> int:
        score = 0
        for i, row in enumerate(x):
            for j, char in enumerate(row):
                if char == target[i][j]:
                    score += 1
        return score

    engine = (
        rd.Engine.char(shape)
        .fitness(fit)
        .select(survivor=rd.Select.elite())
        .alters(rd.Cross.uniform(0.7), rd.Mutate.uniform(0.1))
        .limit(rd.Limit.score(sum(len(t) for t in target)), rd.Limit.generations(2000))
    )

    result = engine.run()

    assert result.value() == [list(t) for t in target]
    assert result.score() == [sum(len(t) for t in target)]
    assert result.index() <= 2000
