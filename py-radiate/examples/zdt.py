#!/usr/bin/env python3
"""
ZDT Multi-Objective Optimization Example

This example demonstrates using the ZDT3 problem for multi-objective optimization.
We define a fitness function based on the ZDT3 problem and use a genetic algorithm to
evolve solutions. The results are visualized in a 2D scatter plot.
"""

import matplotlib.pyplot as plt
import radiate as rd
import numpy as np
from numba import jit, float64

rd.random.seed(501)

variables = rd.random.int(4, 30)
objectives = 2


@jit(float64[:](float64[:]), nopython=True)
def zdt3(val: np.ndarray) -> np.ndarray:
    f1 = val[0]
    g = 1.0 + 9.0 * np.sum(val[1:]) / (variables - 1)
    h = 1.0 - np.sqrt(f1 / g) - (f1 / g) * np.sin(10.0 * np.pi * f1)
    f2 = g * h
    return np.array([f1, f2], dtype=np.float64)


engine = rd.GeneticEngine(
    codec=rd.FloatCodec(variables, use_numpy=True),
    fitness_func=zdt3,
    offspring_selector=rd.TournamentSelector(k=5),
    survivor_selector=rd.NSGA3Selector(points=12),
    objective=["min" for _ in range(objectives)],
    front_range=(200, 250),
    alters=[
        rd.SimulatedBinaryCrossover(1.0, 2.0),
        rd.UniformMutator(0.1),
    ],
)

result = engine.run(rd.GenerationsLimit(2000), ui=True)
print(result.metrics().dashboard())

front = result.front()

x = [member.score()[0] for member in front]
y = [member.score()[1] for member in front]

fig = plt.figure()
ax = plt.axes()
ax.scatter(x, y)
plt.show()
