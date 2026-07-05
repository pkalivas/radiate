#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import radiate as rd

rd.random.seed(67123)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])

engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    .select(rd.Select.boltzmann(temp=4.0))
    .regression(inputs, answers, loss=rd.MSE)
    .alters(
        rd.Cross.graph(0.4, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(rd.Expr.diversity_rate(0.01, 0.9, 20), 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)

result = engine.run(ui=True)

eval_results = result.value().eval(inputs)
accuracy = rd.accuracy(result.value(), inputs, answers, loss=rd.MSE)

print(result)
print(result.metrics().dashboard())
print(accuracy)
print(result.dtype())
