import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_char_vector():
    target = "Testing, Radiate!"

    def fitness_func(x: list[str]) -> int:
        return sum(1 for i in range(len(target)) if x[i] == target[i])

    engine = rd.GeneticEngine(
        codec=rd.CharCodec.vector(len(target)),
        fitness_func=fitness_func,
        offspring_selector=rd.BoltzmannSelector(4),
    )

    result = engine.run([rd.ScoreLimit(len(target)), rd.GenerationsLimit(1000)])

    assert result.value() == list(target)
    assert result.score() == [len(target)]
    assert result.index() <= 1000


@pytest.mark.integration
def test_engine_char_matrix(random_seed):
    shape = [5, 7, 5]
    target = ["Hello", "Radiate", "World"]

    def fitness_func(x: list[list[str]]) -> int:
        score = 0
        for i, row in enumerate(x):
            for j, char in enumerate(row):
                if char == target[i][j]:
                    score += 1
        return score

    engine = rd.GeneticEngine(
        codec=rd.CharCodec.matrix(shape),
        fitness_func=fitness_func,
        survivor_selector=rd.EliteSelector(),
        alters=[rd.UniformCrossover(0.7), rd.UniformMutator(0.1)],
    )

    result = engine.run([rd.ScoreLimit(sum(len(t) for t in target)), rd.GenerationsLimit(2000)])

    assert result.value() == [list(t) for t in target]
    assert result.score() == [sum(len(t) for t in target)]
    assert result.index() <= 2000
