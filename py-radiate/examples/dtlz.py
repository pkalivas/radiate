#!/usr/bin/env python3
"""
DTLZ Multi-Objective Optimization Example

This example demonstrates using the DTLZ1 problem for multi-objective optimization.
We define a fitness function based on the DTLZ1 problem and use a genetic algorithm to
evolve solutions. The results are visualized in a 3D scatter plot.
"""

import matplotlib.pyplot as plt
import radiate as rd
import numpy as np
from numba import jit, float64

rd.random.seed(501)

variables = 4
objectives = 3
k = variables - objectives + 1


@jit(float64[:](float64[:]), nopython=True)
def dtlz_1(val: np.ndarray) -> np.ndarray:
    g_vals = val[variables - k :] - 0.5
    g = 100.0 * (k + np.sum(g_vals**2 - np.cos(20.0 * np.pi * g_vals)))

    base = 0.5 * (1.0 + g)

    f = np.full(objectives, base, dtype=np.float64)

    for i in range(objectives):
        prod_end = objectives - 1 - i
        if prod_end > 0:
            f[i] *= np.prod(val[:prod_end])

        if i > 0:
            f[i] *= 1.0 - val[objectives - 1 - i]

    return f


engine = rd.Engine(
    codec=rd.FloatCodec(variables, use_numpy=True),
    fitness_func=dtlz_1,
    offspring_selector=rd.TournamentSelector(k=5),
    survivor_selector=rd.NSGA3Selector(points=12),
    objective=["min" for _ in range(objectives)],
    front_range=(100, 150),
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
z = [member.score()[2] for member in front]

fig = plt.figure()
ax = plt.axes(projection="3d")
ax.scatter(x, y, z)
ax.set_xlim([0.0, 0.5])
ax.set_ylim([0.0, 0.5])
ax.set_zlim([0.0, 0.5])
plt.show()
