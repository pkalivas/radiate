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
        for i in range(len(chromosome)):
            if rd.random.float() < self.rate:
                chromosome[i].apply(lambda g: {**g, "number": rd.random.float()})
        return chromosome


engine = rd.GeneticEngine(
    codec=rd.AnyCodec(5, lambda: ObjectGene()),
    fitness_func=lambda x: sum(g.number for g in x),
    objectives="min",
    alters=[rd.UniformCrossover(0.5), ObjectMutator(0.1)],
)

print(engine.run(rd.SecondsLimit(4), log=True))
