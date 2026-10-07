import numpy as np
import radiate as rd

rd.random.seed(514)

NUM_BITS = 10000


def fit(val: np.ndarray) -> int:
    return np.bitwise_count(val).sum()


engine = (
    rd.Engine.packed_bit(NUM_BITS, use_numpy=True, words=True)
    .fitness(fit)
    .alter(rd.Cross.multipoint(0.7, 2), rd.Mutate.bit_flip(0.2 / NUM_BITS))
    .limit(rd.Limit.score(NUM_BITS))
)

print(engine.run())
