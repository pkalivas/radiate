"""
This example shows how to use Radiate with NumPy arrays.

It implements a simple feedforward neural network with 3 layers (input, hidden, output) to fit a regression problem.
The network weights are evolved using a float codec.
"""

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
        # Create an engine that evolves genomes with 3 chromosomes, one for each layer's weights, 1 with 16 genes, 1 with 64 genes, and 1 with 8 genes
        shape=[16, 64, 8],
        # Each gene is initialized randomly in the range [-1, 1]
        init_range=(-1.0, 1.0),
        # Genes aren't allowed to go outside the range [-3, 3] during evolution
        bounds=(-3.0, 3.0),
        # Decode radiate's backend (rust) chromosomes into numpy arrays for the fitness function
        use_numpy=True,
        # Use 32-bit floats in radiate's backend (rust side) - note the numpy arrays will also be float32, so we avoid unnecessary up/down casting
        dtype=rd.Float32,
    )
    .fitness(fit)
    .minimizing()
    .select(rd.Select.boltzmann(temp=4.0))
    .alters(rd.Cross.blend(0.7, 0.4), rd.Mutate.gaussian(0.1))
    .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
)

print(engine.run(log=True))
