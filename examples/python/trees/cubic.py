#!/usr/bin/env python3

"""
Regression with Tree Codec

This example demonstrates using the TreeCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import numpy as np
import radiate as rd

rd.random.seed(500)


features, labels = rd.DataSet.cubic().pair()

engine = (
    rd.Engine.tree(
        shape=(1, 1),
        leaf=[rd.Op.var(0), rd.Op.erc()],
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.add(), rd.Op.dot()],
        root=rd.Op.linear(),
    )
    .regression(features, labels)
    .alter(rd.Cross.tree(0.7), rd.Mutate.op())
    .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
)


result = engine.run(log=True)
eval_results = result.value().eval(features)
accuracy = np.mean(
    np.abs(np.array(eval_results).flatten() - np.array(labels).flatten()) < 0.1
)

print(result)
print(f"Accuracy within 0.1: {accuracy * 100:.2f}%")
