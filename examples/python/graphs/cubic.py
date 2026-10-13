#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import numpy as np
import radiate as rd

rd.random.seed(67123)


x, y = rd.DataSet.cubic(dtype=rd.Float32).pair()


def fit(graph: rd.Graph) -> np.float32:
    # Unchecked evaluates the graph without safety checks around
    # the inputs (x) - this can be faster but unsafe if x contains invalid values/shapes.
    predictions = graph.eval(x, unchecked=True)
    return np.mean((predictions - y) ** 2, dtype=np.float32)


engine = (
    rd.Engine.graph(
        # specify the shape of the graph: (num_inputs, num_outputs)
        shape=(1, 1),
        # all vertex nodes will pick a random Op<T> from the below list.
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.sum()],
        # all edge nodes will use this operation - can be a list too
        edge=rd.Op.weight(),
        # specify the output operation for the graph nodes
        output=rd.Op.sum(),
        # specify the dtype of the underlying graph node's Op's dtype T (Op<T>) -
        # input data (x, y) must match this dtype - notice how
        # line 32 specifies the np dtype when computing the fitness.
        dtype=rd.Float32,
    )
    # .fitness(fit)
    # .minimizing()
    # calling regression below is _roughly_ equivalent to setting the fitness function
    # and the objective above. However, the below runs in *pure rust*, is vectorized during evaluation, very cache
    # friendly, and as such is going to be much faster - no need to cross the rust/python bridge.
    .regression(x, y, loss=rd.MSE)
    .select(rd.Select.boltzmann(temp=4.0))
    .alter(
        rd.Cross.graph(0.4, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)


result = engine.run(ui=True)

eval_results = result.value.eval(x)
accuracy = rd.accuracy(result.value, x, y, loss=rd.MSE)

print(result)
print(result.metrics.dashboard())
print(accuracy)
print(result.dtype())

graph = result.value
out = graph.eval([2.2], unchecked=True)
print(type(out), out)
