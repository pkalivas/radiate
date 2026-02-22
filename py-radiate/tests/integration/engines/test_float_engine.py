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
        rd.Engine.float(3, init_range=(-1.0, 1.0))
        .fitness(fitness_func)
        .maximizing()
        .size(50)
        .select(rd.Select.boltzmann(4.0), rd.Select.elite())
        .alters(rd.Cross.uniform(0.7), rd.Mutate.arithmetic(0.01))
    )

    result = engine.run(rd.ScoreLimit(2.9), rd.GenerationsLimit(100))

    # Should find values close to ±1.0
    assert result.score()[0] > 2.5
    assert result.index() <= 100
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == rd.MAX


@pytest.mark.integration
def test_engine_float_matrix_minimization(random_seed):
    """Test engine with float codec for minimization."""

    # Simple fitness function: minimize sum of squares
    def fit(x: list[np.ndarray]) -> float:
        assert isinstance(x, list)
        assert all(isinstance(xi, np.ndarray) and xi.dtype == np.float32 for xi in x)
        return float(np.sum([np.sum(xi**2) for xi in x]))

    # Create an engine that evolves 2x2 matrices of float32 values, minimizing the sum of squares
    engine = (
        rd.Engine.float(
            shape=[2, 2],
            init_range=(-5.0, 5.0),
            bounds=(-5.0, 5.0),
            use_numpy=True,
            dtype=rd.Float32,
        )
        .fitness(fit)
        .minimizing()
        .size(50)
        .select(rd.Select.tournament(3), rd.Select.elite())
        .alters(rd.Cross.mean(0.7), rd.Mutate.gaussian(0.1))
    )

    result = engine.run(rd.ScoreLimit(0.1), rd.GenerationsLimit(200))

    # Should find values close to 0.0
    assert result.score()[0] < 0.5
    assert result.index() <= 200
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == rd.MIN


@pytest.mark.integration
def test_engine_float_simple_neural_network(
    random_seed, example_1x1_regression_dataset
):
    """Test engine with float codec for evolving a simple neural network."""
    inputs, answers = example_1x1_regression_dataset

    X = np.array(inputs, dtype=np.float32)  # (N, 1)
    Y = np.array(answers, dtype=np.float32)  # (N, 1)

    # Add bias term: (N, 2) = [x, 1]
    Xb = np.concatenate([X, np.ones((X.shape[0], 1), dtype=np.float32)], axis=1)

    def fit(weights: list[np.ndarray]) -> float:
        # Decode weights
        W1 = weights[0].reshape((8, 2))
        W2 = weights[1].reshape((8, 8))
        W3 = weights[2].reshape((1, 8))

        # Forward pass
        # Xb: (N,2)
        h1 = Xb @ W1.T  # (N,2) @ (2,8) => (N,8)
        h1 = np.maximum(0, h1)  # ReLU activation

        h2 = h1 @ W2  # (N,8) @ (8,8) => (N,8)
        h2 = np.tanh(h2)  # tanh activation

        yhat = h2 @ W3.T  # (N,8) @ (8,1) => (N,1)

        # MSE
        return float(np.mean((yhat - Y) ** 2, dtype=np.float32))

    engine = (
        rd.Engine.float(
            shape=[16, 64, 8],
            init_range=(-1.0, 1.0),
            bounds=(-3.0, 3.0),
            use_numpy=True,
            dtype=rd.Float32,
        )
        .fitness(fit)
        .minimizing()
        .select(offspring=rd.Select.boltzmann(temp=4.0))
        .alters(rd.Cross.blend(0.7, 0.4), rd.Mutate.gaussian(0.1))
        .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
    )

    result = engine.run()

    # assert result.score()[0] < 0.01
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == rd.MIN
    assert result.dtype() == rd.Float32
    assert len(result.value()) == 3
    assert all(
        isinstance(w, np.ndarray) and w.dtype == np.float32 for w in result.value()
    )
