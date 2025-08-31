#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd


rd.random.seed(42)


class ObjectGene(rd.AnyGene):
    def __init__(self):
        self.number = rd.random.float()
        # self.text = "hello"
        # self.flag = True
        # self.complex = {"key": "value", "list": [rd.random.int(0, 10) for _ in range(3)]}

    def __repr__(self):
        return f"ObjectGene(number={self.number}"#, text={self.text}, flag={self.flag}, complex={self.complex})"

codec = rd.AnyCodec(2, lambda: ObjectGene())

c = codec.encode()

# for g in c:
#     for o in g:
#         print(o)

def fitness_function(individuals):
    return abs(sum(individual.number for individual in individuals) - 4)
    # return abs(sum(g for ind in individuals for g in ind.complex['list']) - 4)

engine = rd.GeneticEngine(
    codec=rd.AnyCodec(5, lambda: ObjectGene()),
    fitness_func=fitness_function,
    objectives="min",
    alters=[
        rd.FieldAlterer.swap("number", rate=0.5),
        rd.FieldAlterer.uniform("number", rate=0.1, bounds=(-0.1, 1.0)),
        # rd.FieldAlterer.jitter("number", rate=0.1, amount=0.1),

        # rd.FieldAlterer.swap("list", rate=0.5),
        # rd.UniformCrossover(0.5),
    ],
    # executor=rd.Executor.FixedSizedWorkerPool(4),
)

print(engine.run([rd.ScoreLimit(0.0001), rd.SecondsLimit(4)], log=True))


codec = rd.AnyCodec(2, lambda: ObjectGene())

c = codec.encode()

for g in c:
    for o in g:
        print(o)

# print(ObjectGene().__backend__().number)
