#!/usr/bin/env python3
"""
PyTorch Neural Network Weight Evolution with Radiate FloatCode
"""

import radiate as rd
import torch  # type: ignore
from torch import nn
import torch.nn.functional as F  # type: ignore
import numpy as np


rd.random.seed(42)
torch.manual_seed(42)
np.random.seed(42)


class NN(nn.Module):
    def __init__(
        self,
    ):
        super(NN, self).__init__()
        self.fc1 = nn.Linear(2, 4)
        self.fc2 = nn.Linear(4, 1)

    def forward(self, x):
        x = F.relu(self.fc1(x))
        x = self.fc2(x)
        return x


X = torch.tensor([[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]], dtype=torch.float64)
y = torch.tensor([[0.0], [1.0], [1.0], [0.0]], dtype=torch.float64)

network = NN()


def fit(weights: list[np.ndarray]) -> float:
    idx = 0
    for param in network.parameters():
        param.data = torch.from_numpy(weights[idx].reshape(param.shape))
        idx += 1

    predictions = network(X)
    mse = F.mse_loss(predictions, y)
    return mse.item()


engine = (
    rd.Engine.float(
        shape=[layer.numel() for layer in network.state_dict().values()],
        init_range=(-2.0, 2.0),
        bounds=(-5.0, 5.0),
        use_numpy=True,
    )
    .fitness(fit)
    .minimizing()
    .select(survivor=rd.BoltzmannSelector(temp=3))
    .alters(
        rd.BlendCrossover(0.7, 0.5),
        rd.GaussianMutator(0.05),
    )
)


print("Starting Neural Network Weight Evolution")
print("=" * 50)
print(f"Target error: {0.01}")
print(f"Max generations: {500}")
print()
result = engine.run(rd.ScoreLimit(0.01), rd.GenerationsLimit(500), log=True)
best_weights = result.value()
print("\n" + "=" * 50)
print("BEST EVOLVED NEURAL NETWORK")
print("=" * 50)
with torch.no_grad():
    idx = 0
    for name, param in network.named_parameters():
        param.data = torch.tensor(best_weights[idx], dtype=torch.float64).view(
            param.shape
        )
        idx += 1
print("XOR Problem Results:")
print("Input\t\tExpected\tPredicted\tError")
print("-" * 50)
total_error = 0.0
for i in range(len(X)):
    input_data = X[i : i + 1]  # Add batch dimension
    target = y[i].item()
    prediction = network(input_data)
    predicted = prediction.item()
    error = abs(target - predicted)
    total_error += error
    print(f"{X[i].tolist()}\t\t{target}\t\t{predicted:.4f}\t\t{error:.4f}")
print("-" * 50)
print(f"Average Error: {total_error / len(X):.4f}")
print(f"Network successfully learned XOR: {total_error / len(X) < 0.1}")
