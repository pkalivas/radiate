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
from numba import jit, float32

rd.random.seed(501)

variables = 4
objectives = 3
k = variables - objectives + 1


@jit(float32[:](float32[:]), nopython=True)
def dtlz_1(val: np.ndarray) -> np.ndarray:
    g_vals = val[variables - k :] - 0.5
    g = 100.0 * (k + np.sum(g_vals**2 - np.cos(20.0 * np.pi * g_vals)))

    base = 0.5 * (1.0 + g)

    f = np.full(objectives, base, dtype=np.float32)

    for i in range(objectives):
        prod_end = objectives - 1 - i
        if prod_end > 0:
            f[i] *= np.prod(val[:prod_end])

        if i > 0:
            f[i] *= 1.0 - val[objectives - 1 - i]

    return f


engine = rd.GeneticEngine(
    codec=rd.FloatCodec.vector(variables, (0.0, 1.0), (-2.0, 2.0), use_numpy=True),
    fitness_func=dtlz_1,
    offspring_selector=rd.TournamentSelector(k=8),
    survivor_selector=rd.NSGA2Selector(),
    objective=["min" for _ in range(objectives)],
    alters=[
        rd.SimulatedBinaryCrossover(1.0, 2.0),
        rd.UniformMutator(0.1),
    ],
)

result = engine.run(rd.GenerationsLimit(2000), log=True)
print(result)

front = result.value()

fig = plt.figure()
ax = plt.axes(projection="3d")

x = [member["fitness"][0] for member in front]
y = [member["fitness"][1] for member in front]
z = [member["fitness"][2] for member in front]

ax.scatter(x, y, z)
ax.set_xlim([0, 0.5])
ax.set_ylim([0, 0.5])
ax.set_zlim([0, 0.5])
plt.show()
