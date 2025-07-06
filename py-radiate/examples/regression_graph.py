#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import radiate as rd

rd.random.set_seed(518)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])

engine = rd.GeneticEngine(
    codec=rd.GraphCodec.directed(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    ),
    fitness_func=rd.Regression(inputs, answers),
    objectives="min",
    alters=[
        rd.GraphCrossover(0.5, 0.5),
        rd.OperationMutator(0.07, 0.05),
        rd.GraphMutator(0.1, 0.1),
    ],
)

print(engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(1000)], log=True))
