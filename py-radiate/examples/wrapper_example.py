#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd
from datetime import datetime, timezone

rd.random.seed(42)


class ObjectGene(rd.AnyGene):
    def __init__(self):
        self.number = rd.random.int(min=0, max=10)
        self.date = datetime(2020, 1, 1, tzinfo=timezone.utc)

    def __repr__(self):
        return f"ObjectGene(number={self.number}, date={self.date})"


def fitness_function(phenotypes: list[list[ObjectGene]]) -> list[float]:
    return [sum(gene.number for gene in individual) for individual in phenotypes]


engine = rd.GeneticEngine(
    rd.AnyCodec(ObjectGene() for _ in range(10)),
    fitness_func=rd.BatchFitness(fitness_function),
    objective="min",
)

result = engine.run(rd.ScoreLimit(0), ui=True)

for obj_gene in result.value():
    print(obj_gene)
