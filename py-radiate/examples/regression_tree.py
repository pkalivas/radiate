#!/usr/bin/env python3

import radiate as rd

rd.random.set_seed(500)

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
    codec=rd.TreeCodec(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.add()],
        root=rd.Op.linear(),
    ),
    fitness_func=rd.Regression(inputs, answers),
    objectives="min",
    alters=[
        rd.TreeCrossover(0.7),
        rd.HoistMutator(0.01),
    ],
)

if __name__ == "__main__":
    result = engine.run([rd.ScoreLimit(0.01), rd.GenerationsLimit(1000)], log=True)
    print(result)
    print(result.value().__repr__())
