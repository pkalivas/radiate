#!/usr/bin/env python3

"""
Regression with Tree Codec

This example demonstrates using the TreeCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.

The final result (most optimal tree) is this:

└── Fn:   sum
    ├── Fn:   mul
    │   ├── Fn:   add
    │   │   ├── Fn:   add
    │   │   │   ├── Var:  X0(0)
    │   │   │   └── Var:  X0(0)
    │   │   └── Fn:   sub
    │   │       ├── Var:  X0(0)
    │   │       └── Var:  X0(0)
    │   └── Fn:   sub
    │       ├── Fn:   mul
    │       │   ├── Fn:   add
    │       │   │   ├── Var:  X0(0)
    │       │   │   └── Var:  X0(0)
    │       │   └── Var:  X0(0)
    │       └── Var:  X0(0)
    └── Fn:   sub
        ├── Var:  X0(0)
        └── Fn:   mul
            ├── Var:  X0(0)
            └── Var:  X0(0)

which when reduced is: f(x) = 4x^3 - 3x^2 + x, which is the exact cubic function used to
create the dataset.
"""

import numpy as np
import radiate as rd

rd.random.seed(500)


features, labels = rd.DataSet.cubic().pair()

engine = (
    rd.Engine.tree(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.add()],
        root=rd.Op.sum(),
    )
    .regression(features, labels)
    .alter(rd.Cross.tree(0.7), rd.Mutate.op())
    .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
)


result = engine.run(log=True)
eval_results = result.value.eval(features)
accuracy = np.mean(
    np.abs(np.array(eval_results).flatten() - np.array(labels).flatten()) < 0.1
)

print(result)
print(f"Accuracy within 0.1: {accuracy * 100:.2f}%")

print(result.value.format())
