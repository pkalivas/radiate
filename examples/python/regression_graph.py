#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import numpy as np
import radiate as rd

# rd.random.seed(67123)
rd.random.seed(67)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])

x = np.array(inputs, dtype=np.float64)
y = np.array(answers, dtype=np.float64)


def fit(graph: rd.Graph) -> np.float64:
    predictions = graph.eval(x)
    return np.mean((predictions - y) ** 2, dtype=np.float64)


# @rd.fitness(batch=True)
# def fit(graph: list[rd.Graph]) -> list[np.float32]:
#     # raise NotImplementedError("This function is not implemented yet.")
#     predictions = [g.eval(x) for g in graph]
#     return [np.mean((pred - y) ** 2, dtype=np.float32) for pred in predictions]


engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
        dtype=rd.Float64,
        # output=rd.Op.linear(),
    )
    .fitness(fit)
    .minimizing()
    # .regression(x, y, loss=rd.MSE)
    .select(rd.Select.boltzmann(temp=4.0))
    .alters(
        rd.Cross.graph(0.4, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)


result = engine.run(log=True)

eval_results = result.value().eval(x)
accuracy = rd.accuracy(result.value(), x, y, loss=rd.MSE)

print(result)
print(result.metrics().dashboard())
print(accuracy)
print(result.dtype())
