import numpy as np
import radiate as rd

rd.random.seed(514)


A = 10.0
RANGE = 5.12
N_GENES = 2
TARGET = 1e-6


@rd.fitness(batch=True)
def fit(population: np.ndarray) -> np.ndarray:
    return A * N_GENES + np.sum(
        population**2 - A * np.cos(2.0 * np.pi * population),
        axis=1,
    )


engine = (
    rd.Engine.float(shape=N_GENES, init_range=(-RANGE, RANGE), use_numpy=True)
    .fitness(fit)
    .minimizing()
    .alter(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.01))
    .limit(rd.Limit.score(TARGET), rd.Limit.generations(1000))
)

print(engine.run())
