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


class ObjectMutator(rd.Mutator):
    def __init__(self, rate):
        super().__init__(rate)

    def mutate(self, chromosome: rd.Chromosome) -> rd.Chromosome:
        for gene in chromosome:
            if rd.random.float() < self.rate:
                gene.apply(
                    lambda g: {**g, "number": g["number"] + rd.random.float(-0.1, 0.1)}
                )
        return chromosome


engine = rd.GeneticEngine(
    codec=rd.AnyCodec(5, lambda: ObjectGene()),
    fitness_func=lambda x: abs(sum(g.number for g in x) - 4),
    objectives="min",
    alters=[rd.UniformCrossover(0.5), ObjectMutator(0.1)],
)

print(engine.run([rd.ScoreLimit(0.0001), rd.SecondsLimit(4)], log=True))


codec = rd.AnyCodec(1, lambda: ObjectGene())

print(ObjectGene.__newinstance__().__backend__())
print(codec.encode())
print(codec.decode(codec.encode()))

# print(rd.gene.any(ObjectGene()).__backend__())
