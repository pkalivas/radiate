#!/usr/bin/env python3
"""
Example of using radiate to solve the PyGAD example problem found on their homepage.

https://pygad.readthedocs.io/en/latest/

This example should resolve within ~5-10 generations, or a little less than 1ms
depending on your random seed & your machine.
"""

import numpy as np
import radiate as rd
from numba import float64, jit

rd.random.seed(123)

function_inputs = np.array([4.0, -2.0, 3.5, 5.0, -11.0, -4.7])
desired_output = 44.0


@jit(float64(float64[:]), nopython=True)
def fitness(solution: np.ndarray) -> float:
    output = np.sum(solution * function_inputs)
    return np.abs(output - desired_output)


engine = (
    rd.Engine.float(len(function_inputs), init_range=(-4.0, 4.0), use_numpy=True)
    .fitness(fitness)
    .minimizing()
)

result = engine.run(rd.Limit.score(0.01), log=True)

print(f"\nBest solution found: {result.value()}")
print(f"Fitness: {result.score()}")
print(f"Generations completed: {result.index()}")
print(f"Function output: {np.sum(result.value() * function_inputs)}")
print(f"Duration: {result.duration()}")
print(f"dtype: {result.dtype()}")
print(result.metrics().dashboard())


# def fit(val: np.ndarray) -> int:
#     # print(val)
#     # print()

#     # one = val[0].reshape((3, 3))
#     # two = val[1].reshape((3, 3))
#     # three = val[2].reshape((3, 3))

#     print(val)
#     return int(np.sum(val))

#     # return int(np.sum(val[0]) + np.sum(val[1]) + np.sum(val[2]))


# engine = (
#     rd.Engine.float(9, init_range=(-4.0, 4.0), use_numpy=True, dtype=rd.Float32)
#     .fitness(fit)
#     .minimizing()
# )

# engine.run(rd.Limit.generations(1), log=True)
