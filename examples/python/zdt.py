#!/usr/bin/env python3
"""
ZDT Multi-Objective Optimization Example

This example demonstrates how to use the Radiate library to solve a multi-objective optimization
problem using the ZDT3 benchmark function. The ZDT3 function is a commonly used test
problem in multi-objective optimization, which has two objectives and a non-convex Pareto front.
"""

import numpy as np  # type: ignore
import plotly.graph_objects as go
from numba import float64, jit  # type: ignore

import radiate as rd

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


engine = (
    rd.Engine.float(variables, use_numpy=True)
    .fitness(zdt3)
    .objective(rd.MIN, rd.MIN, front_range=(200, 250))
    # NSGA-II: crowded-comparison tournament for parents, rank + crowding distance for survivors.
    # A lower offspring fraction keeps more of the already-evaluated front each generation.
    .select(rd.Select.tournament_nsga2(), rd.Select.nsga2(), frac=0.5)
    .alter(rd.Cross.sbx(0.8, 20.0), rd.Mutate.polynomial(0.1, 20.0))
    .limit(rd.Limit.generations(2000))
)

result = engine.run(ui=True)
print(result.metrics().dashboard())

front = result.front()

x = [member.score()[0] for member in front]
y = [member.score()[1] for member in front]

fig = go.Figure(go.Scatter(x=x, y=y, mode="markers"))
fig.show()
