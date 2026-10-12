#!/usr/bin/env python3
"""
NEAT evolution example using radiate.
"""

import radiate as rd

rd.random.seed(514)

data = rd.DataSet.koza1(dtype=rd.Float32)
_, in_dim, out_dim = data.shape
features, targets = data.pair()


engine = (
    rd.Engine.graph(
        # specify the shape of the graph: (num_inputs, num_outputs)
        shape=(in_dim, out_dim),
        # each vertex node will pick a random Op<T> from the below list.
        vertex=[rd.Op.add(), rd.Op.mul()],
        # all edge nodes will use this operation - can be a list too
        edge=rd.Op.identity(),
        # all output nodes will use this operation - can be a list too
        output=rd.Op.sum(),
        # specify the dtype of the underlying graph node's Op's dtype T (Op<T>) -
        # input data (inputs, outputs) must match this dtype
        dtype=rd.Float32,
    )
    .regression(features, targets, loss=rd.MSE)
    .diversity(
        rd.Dist.neat(excess=1.0, disjoint=1.0, weight_diff=3.0),
        target=4,
    )
    .alter(
        rd.Cross.graph(0.4, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)

result = engine.run(ui=True)

eval_results = result.value().eval(features)
accuracy = rd.accuracy(result.value(), features, targets, loss=rd.MSE)

print(result)
print(result.metrics().dashboard())
print(accuracy)
print(f"Graph Dtype: {result.value().dtype()}")
print("Species count mean:", result.metrics()["species.count"].mean())
