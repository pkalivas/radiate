#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd


rd.random.seed(42)


class ObjectGene(rd.AnyGene):
    def __init__(self):
        self.number = rd.random.int(min=0, max=10)

    def __repr__(self):
        return f"ObjectGene(number={self.number})"


engine = rd.GeneticEngine(
    rd.AnyCodec(ObjectGene() for _ in range(10)),
    lambda x: sum(gene.number for gene in x),
    objectives="min",
)

result = engine.run(rd.ScoreLimit(0), log=True)

for obj_gene in result.value():
    print(obj_gene)

