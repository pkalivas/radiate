#!/usr/bin/env python3
import radiate as rd
import numpy as np

rd.random.seed(123)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

x = -1.0
for _ in range(20):
    x += 0.1
    inputs.append([x])
    answers.append([compute(x)])

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

print(engine.run())
