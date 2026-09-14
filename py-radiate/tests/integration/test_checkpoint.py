from pathlib import Path

import numpy as np
import pytest

import radiate as rd


@pytest.mark.integration
def test_load_checkpoint(example_1x1_regression_dataset, random_seed):
    inputs, answers = example_1x1_regression_dataset
    path = Path(__file__).parent.parent / "data" / "chckpnt_50.json"

    X = np.array(inputs, dtype=np.float32)  # (N, 1)
    Y = np.array(answers, dtype=np.float32)  # (N, 1)
    Xb = np.concatenate([X, np.ones((X.shape[0], 1), dtype=np.float32)], axis=1)

    def fit(weights: list[np.ndarray]) -> float:
        W1 = weights[0].reshape((8, 2))
        W2 = weights[1].reshape((8, 8))
        W3 = weights[2].reshape((1, 8))

        h1 = Xb @ W1.T
        h1 = np.maximum(0, h1)

        h2 = h1 @ W2
        h2 = np.tanh(h2)

        yhat = h2 @ W3.T

        return float(np.mean((yhat - Y) ** 2, dtype=np.float32))

    engine = (
        rd.Engine.float(
            shape=[16, 64, 8],
            init_range=(-1.0, 1.0),
            bounds=(-3.0, 3.0),
            use_numpy=True,
            dtype=rd.Float32,
        )
        .load_checkpoint(path)
        .fitness(fit)
        .minimizing()
        .select(rd.Select.boltzmann(temp=4.0))
        .alter(rd.Cross.blend(0.7, 0.4), rd.Mutate.gaussian(0.1))
        .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
    )

    for epoch in engine:
        assert epoch.index() > 50


@pytest.mark.integration
def test_from_generations(random_seed):
    def fitness(individual: list[list[bool]]) -> float:
        return float(np.sum(individual))

    engine_one = (
        rd.Engine.bit([20, 20])
        .fitness(fitness)
        .minimizing()
        .limit(rd.Limit.generations(20))
    )

    result = engine_one.run()
    first_member = result.population().phenotypes()[0]
    first_score = first_member.score()

    assert result.index() == 20

    other_engine = (
        rd.Engine.bit([20, 20])
        .fitness(fitness)
        .generation(result)
        .minimizing()
        .limit(rd.Limit.score(0))
    )

    for epoch in other_engine:
        assert epoch.score() <= first_score
