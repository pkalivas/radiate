#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import numpy as np
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

x = np.array(inputs, dtype=np.float32)
y = np.array(answers, dtype=np.float32)


def fit(graph: rd.Graph) -> np.float32:
    predictions = graph.eval(x, unchecked=True)
    return np.mean((predictions - y) ** 2, dtype=np.float32)


engine = (
    rd.Engine.graph(
        # specify the shape of the graph: (num_inputs, num_outputs)
        shape=(1, 1),
        # all vertex nodes will pick a random Op<T> from the below list.
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        # all edge nodes will use this operation - can be a list too
        edge=rd.Op.weight(),
        # specify the dtype of the underlying graph node's Op's dtype T (Op<T>) -
        # input data (x, y) must match this dtype
        dtype=rd.Float32,
    )
    # .fitness(fit)
    # .minimizing()
    # calling regression below is _roughly_ equivalent to setting the fitness function
    # and the objective above. However, the below runs in *pure rust* and as such is
    # going to be much faster - no need to cross the rust/python bridge.
    .regression(x, y, loss=rd.MSE)
    .select(rd.Select.boltzmann(temp=4.0))
    .alter(
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

graph = result.value()
out = graph.eval([2.2], unchecked=True)
print(type(out), out)
