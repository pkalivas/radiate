import numpy as np
import radiate as rd

rd.random.seed(514)

TARGET_NUM = 1000


def fit(val: np.ndarray) -> int:
    return np.bitwise_count(val).sum()


engine = (
    rd.Engine.packed_bit(TARGET_NUM, use_numpy=True, words=True)
    .fitness(fit)
    .alter(rd.Cross.multipoint(0.7, 2), rd.Mutate.bit_flip(0.02 / TARGET_NUM))
    .limit(rd.Limit.score(TARGET_NUM))
)

print(engine.run())
