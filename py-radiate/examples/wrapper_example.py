#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd


rd.random.seed(42)


class ObjectGene(rd.AnyGene):
    def __init__(self):
        self.number = rd.random.float()

    def __repr__(self):
        return f"ObjectGene(number={self.number})"


engine = rd.GeneticEngine(
    codec=rd.AnyCodec(5, lambda: ObjectGene()),
    fitness_func=lambda x: abs(sum(g.number for g in x) - 4),
    objectives="min",
    # alters=[
    #     rd.UniformCrossover(0.5),
    #     ObjectMutator(0.1),
    # ],
    executor=rd.Executor.FixedSizedWorkerPool(4),
)

print(engine.run([rd.ScoreLimit(0.0001), rd.SecondsLimit(4)], log=True))


codec = rd.AnyCodec(1, lambda: ObjectGene())

# print(ObjectGene().__backend__().number)
