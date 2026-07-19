#!/usr/bin/env python3
"""
NEAT evolution. This example shows how we can use radiate to evolve
graphs using the NEAT algorithm. Notice the only real difference in the
engine construction is the addition of the NEAT distance function and species
threshold in the `.diversity(...)` method.
"""

import numpy as np
import radiate as rd

rd.random.seed(514)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])

inputs = np.array(inputs, dtype=np.float32)  # (N, 1)
answers = np.array(answers, dtype=np.float32)  # (N, 1)

engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
        dtype=rd.Float32,  # specify the dtype of the underlying graph node's Op's dtype T (Op<T>) - input data (X, Y) must match this dtype
    )
    .select(rd.Select.boltzmann(temp=4.0))
    # .filter(rd.Filter.unique_score()). # uncomment to filter out phenotypes with duplicate scores each generation
    .regression(inputs, answers, loss=rd.MSE)
    .diversity(
        rd.Dist.neat(excess=1.0, disjoint=1.0, weight_diff=3.0),
        target=5,
    )
    .alters(
        rd.Cross.graph(0.4, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)

result = engine.run(ui=True)

eval_results = result.value().eval(inputs)
accuracy = rd.accuracy(result.value(), inputs, answers, loss=rd.MSE)

print(result)
print(result.metrics().dashboard())
print(accuracy)
print(f"Graph Dtype: {result.value().dtype()}")
