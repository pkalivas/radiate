#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = [
#   "torch",
#   "numpy",
# ]
# ///
"""
PyTorch Neural Network Weight Evolution with Radiate FloatCode
"""

# pyright: reportMissingImports=false

import os
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import radiate as rd
import torch
from torch import nn
import torch.nn.functional as F
import numpy as np


rd.random.seed(42)
torch.manual_seed(42)
np.random.seed(42)


class NN(nn.Module):
    def __init__(self, weights: list[np.ndarray] | None = None):
        super(NN, self).__init__()
        self.fc1 = nn.Linear(2, 4)
        self.fc2 = nn.Linear(4, 1)

        if weights is not None:
            self.set_weights(weights)

    def forward(self, x):
        x = F.relu(self.fc1(x))
        x = self.fc2(x)
        return x

    def set_weights(self, weights: list[np.ndarray]):
        for param, weight in zip(self.parameters(), weights):
            param.data = torch.from_numpy(weight.reshape(param.shape))


X = torch.tensor([[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]], dtype=torch.float64)
y = torch.tensor([[0.0], [1.0], [1.0], [0.0]], dtype=torch.float64)

network = NN()
network.eval()


def fit(weights: list[np.ndarray]) -> float:
    with torch.no_grad():
        network.set_weights(weights)
        predictions = network(X)
        mse = F.mse_loss(predictions, y)
        return mse.item()


engine = (
    rd.Engine.float(
        shape=[layer.numel() for layer in NN().state_dict().values()],
        init_range=(-2.0, 2.0),
        bounds=(-5.0, 5.0),
        use_numpy=True,
    )
    .fitness(fit)
    .minimizing()
    .select(offspring=rd.Select.boltzmann(temp=3))
    .alters(
        rd.Cross.blend(0.7, 0.5),
        rd.Mutate.gaussian(0.05),
    )
    .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
)


result = engine.run(log=True)
best_weights = result.value()

with torch.no_grad():
    network = NN(best_weights)
    network.eval()

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
