#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd


rd.random.seed(42)


@rd.any_gene
class TestGene:
    def __init__(self):
        self.name = "TestGene"
        self.number = rd.random.randfloat()

    def __repr__(self):
        return f"TestGene(name={self.name}, number={self.number})"


class Muta(rd.Mutator):
    def __init__(self, rate):
        super().__init__(rate)

    def mutate(self, chromosome: rd.Chromosome) -> rd.Chromosome:
        for i in range(len(chromosome)):
            if rd.random.randfloat() < self.rate:
                chromosome[i].apply(lambda g: {**g, "number": rd.random.randfloat()})
        return chromosome


engine = rd.GeneticEngine(
    codec=rd.AnyCodec(5, lambda: TestGene()),
    fitness_func=lambda x: sum(g.number for g in x),
    objectives="min",
    alters=[rd.UniformCrossover(0.5), rd.UniformMutator(0.1)],
)

print(engine.run(rd.SecondsLimit(4), log=True))
