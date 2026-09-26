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


engine = (
    rd.Engine.float(variables, use_numpy=True)
    .fitness(dtlz_1)
    .objective(rd.MIN, rd.MIN, rd.MIN)
    .front_range(100, 150)
    # NSGA-III for 3+ objectives: crowded-comparison tournament for parents, reference-point
    # NSGA-II also works honestly just fine for this example
    # niching for survivors. A lower offspring fraction keeps more of the evaluated front.
    .select(
        rd.Select.tournament_nsga2(),
        # rd.Select.nsga3(points=12),
        rd.Select.nsga2(),
        frac=0.5,
    )
    .alter(
        rd.Cross.sbx(0.8, 20.0),
        rd.Mutate.polynomial(0.1, 20.0),
    )
    .limit(rd.Limit.generations(2000))
)


result = engine.run(ui=True)
front = result.front()
hypervolume = front.hypervolume([1.0, 1.0, 1.0])

print(result.metrics().dashboard())
print()
# DTLZ1's Pareto front is the plane f1 + f2 + f3 = 0.5, so with a reference point of
# 1.0 in every objective the best achievable hypervolume is 1 - 0.5³/6 ≈ 0.9792.
print(f"Hypervolume: {hypervolume}")

x = [member.score()[0] for member in front]
y = [member.score()[1] for member in front]
z = [member.score()[2] for member in front]

fig = go.Figure(go.Scatter3d(x=x, y=y, z=z, mode="markers"))
fig.update_layout(
    scene={
        "xaxis": {"range": [0.0, 0.5]},
        "yaxis": {"range": [0.0, 0.5]},
        "zaxis": {"range": [0.0, 0.5]},
    }
)
fig.show()
