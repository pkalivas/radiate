#!/usr/bin/env python3
"""
Example of using radiate to solve the PyGAD example problem found on their homepage.

https://pygad.readthedocs.io/en/latest/

This example should resolve within ~5-10 generations, or a little less than 1ms
depending on your random seed & your machine.
"""

import numpy as np
import radiate as rd
from numba import float32, jit

rd.random.seed(42)

function_inputs = np.array([4.0, -2.0, 3.5, 5.0, -11.0, -4.7])
desired_output = 44.0


@jit(float32(float32[:]), nopython=True)
def fitness(solution: np.ndarray) -> float:
    output = np.sum(solution * function_inputs)
    return np.abs(output - desired_output)


engine = rd.GeneticEngine(
    codec=rd.FloatCodec.vector(len(function_inputs), (-4.0, 4.0), use_numpy=True),
    fitness_func=fitness,
    objective="min",
)

result = engine.run(rd.ScoreLimit(0.01), log=True)

print(f"\nBest solution found: {result.value()}")
print(f"Fitness: {result.score()}")
print(f"Generations completed: {result.index()}")
print(f"Function output: {np.sum(result.value() * function_inputs)}")
print(f"Duration: {result.duration()}")

print(result.metrics().dashboard())
