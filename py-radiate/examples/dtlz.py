#!/usr/bin/env python3
"""
DTLZ Multi-Objective Optimization Example

This example demonstrates using the DTLZ1 problem for multi-objective optimization.
We define a fitness function based on the DTLZ1 problem and use a genetic algorithm to
evolve solutions. The results are visualized in a 3D scatter plot.
"""

from typing import List
import math
import matplotlib.pyplot as plt  # type: ignore
import radiate as rd

rd.random.set_seed(500)

variables = 4
objectives = 3
k = variables - objectives + 1


def dtlz_1(val: List[float]) -> List[float]:
    g = 0.0
    for i in range(variables - k, variables):
        g += (val[i] - 0.5) ** 2 - math.cos(20.0 * math.pi * (val[i] - 0.5))
    g = 100.0 * (k + g)
    f = [0.0] * objectives
    for i in range(objectives):
        f[i] = 0.5 * (1.0 + g)
        for j in range(objectives - 1 - i):
            f[i] *= val[j]
        if i != 0:
            f[i] *= 1.0 - val[objectives - 1 - i]
    return f


engine = rd.GeneticEngine(
    codec=rd.FloatCodec.vector(variables, (0.0, 1.0), (-2.0, 2.0)),
    fitness_func=dtlz_1,
    offspring_selector=rd.TournamentSelector(k=8),
    survivor_selector=rd.NSGA2Selector(),
    objectives=["min" for _ in range(objectives)],
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

ax.scatter(x, y, z, c="r", marker="o")
ax.set_xlim([0, 0.5])
ax.set_ylim([0, 0.5])
ax.set_zlim([0, 0.5])
plt.show()
