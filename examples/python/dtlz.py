#!/usr/bin/env python3
"""
DTLZ Multi-Objective Optimization Example

This example demonstrates using the DTLZ1 problem for multi-objective optimization.
We define a fitness function based on the DTLZ1 problem and use a genetic algorithm to
evolve solutions. The results are visualized in a 3D scatter plot.
"""

import numpy as np  # type: ignore
import plotly.graph_objects as go
import radiate as rd
from numba import float64, jit  # type: ignore

rd.random.seed(501)

variables = 4
objectives = 3
k = variables - objectives + 1


@rd.fitness(batch=True)
@jit(float64[:, :](float64[:, :]), nopython=True)
def fit(vals: np.ndarray) -> np.ndarray:
    g_vals = vals[:, variables - k :] - 0.5
    g = 100.0 * (k + np.sum(g_vals**2 - np.cos(20.0 * np.pi * g_vals), axis=1))

    f = np.empty((vals.shape[0], objectives), dtype=np.float64)
    for i in range(objectives):
        f[:, i] = 0.5 * (1.0 + g)
        for j in range(objectives - 1 - i):  # numba's np.prod has no axis argument
            f[:, i] *= vals[:, j]
        if i > 0:
            f[:, i] *= 1.0 - vals[:, objectives - 1 - i]
    return f


engine = (
    rd.Engine.float(variables, use_numpy=True)
    .fitness(fit)
    .objective(rd.MIN, rd.MIN, rd.MIN)
    .front_range(200, 250)
    # NSGA-III for 3+ objectives: crowded-comparison tournament for parents, reference-point
    # rd.Select.nsga3(points=12),
    # NSGA-II also works honestly just fine for this example
    # niching for survivors. A lower offspring fraction keeps more of the evaluated front.
    .select(
        rd.Select.tournament_nsga2(),
        rd.Select.nsga2(),
        frac=0.5,
    )
    .alter(
        rd.Cross.sbx(0.5, 20.0),
        rd.Cross.differential(rate=0.8, weight=0.8, crossover_rate=0.9),
        rd.Mutate.polynomial(0.1, 20.0),
    )
    .limit(rd.Limit.generations(2000))
)

result = engine.run(ui=True)
hypervolume = result.front.hypervolume([1.0, 1.0, 1.0])

print(result.metrics.dashboard())
print()
print(f"Hypervolume: {hypervolume}")

x = [member.score[0] for member in result.front]
y = [member.score[1] for member in result.front]
z = [member.score[2] for member in result.front]

axis = {"range": [-0.025, 0.525]}
fig = go.Figure(go.Scatter3d(x=x, y=y, z=z, mode="markers", marker={"size": 4}))
fig.update_layout(
    scene={"xaxis": axis, "yaxis": axis, "zaxis": axis, "aspectmode": "cube"}
)
fig.show()
