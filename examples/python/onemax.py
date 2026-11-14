# !/usr/bin/env python3
"""
OneMax problem - maximize the number of 1s in a binary string.

This is a pretty classic example of a simple optimization problem. I'd also flag this as an important example
due to how the fitness problem and codec are setup. The fitness function flags `batch=True` meaning we
want (will receive) a batch of candidate solutions all at once. At the same time, in the codec, we set
`use_numpy=True`. The combindation of these two parameters, mean radiate's internal processing will
produce a single numpy array conntaining all candidate solutions - in this case a 2D array
where each row is a candidate. This allows us to process the fitness function in one, vectorized operation.
"""

import numpy as np
import radiate as rd

rd.random.seed(514)

NUM_BITS = 1000


@rd.fitness(batch=True)
def fit(val: np.ndarray) -> np.ndarray:
    return np.bitwise_count(val).sum(axis=1)


engine = (
    rd.Engine.packed_bit(NUM_BITS, use_numpy=True, words=True)
    .fitness(fit)
    .alter(rd.Cross.multipoint(0.5, 2), rd.Mutate.bit_flip(0.2 / NUM_BITS))
    .limit(rd.Limit.score(NUM_BITS))
)

print(engine.run().metrics.dashboard())
