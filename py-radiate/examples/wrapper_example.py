#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd


rd.random.seed(42)


class ObjectGene(rd.AnyGene):
    def __init__(self):
        self.number = rd.random.float()
        self.text = "hello"
        self.flag = True
        self.complex = {
            "key": "value",
            "list": [rd.random.int(0, 10) for _ in range(10)],
        }

    def __repr__(self):
        return f"ObjectGene(number={self.number}, text={self.text}, flag={self.flag}, complex={self.complex})"


def fitness_function(individuals):
    number_sum = abs(sum(individual.number for individual in individuals))
    list_sum = abs(sum(g for ind in individuals for g in ind.complex["list"]))
    return list_sum + number_sum


engine = rd.GeneticEngine(
    codec=rd.AnyCodec(5, lambda: ObjectGene()),
    fitness_func=fitness_function,
    objectives="min",
    alters=[
        rd.AnyAlterer.two_point("list", rate=0.5),
        rd.AnyAlterer.uniform("list", rate=0.1, bounds=(-10, 10)),
        rd.AnyAlterer.jitter("number", rate=0.1, amount=0.1),
        # rd.FieldAlterer.swap("list", rate=0.5),
        # rd.UniformCrossover(0.5),
    ],
)

result = engine.run([rd.ScoreLimit(0.0001), rd.SecondsLimit(4)], log=True)


t = result.value()

number_sum = 0
list_sum = 0

for individual in t:
    number_sum += abs(individual.number)
    list_sum += abs(sum(individual.complex["list"]))

print(t)
print(fitness_function(t))

codec = rd.AnyCodec(2, lambda: ObjectGene())

c = codec.encode()

for g in c:
    for o in g:
        print(o)
