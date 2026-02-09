#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd
from datetime import datetime, timezone
import numpy as np

rd.random.seed(42)


class ObjectGene(rd.AnyGene):
    def __init__(self):
        self.number = rd.random.int(min=0, max=10)
        # self.date = datetime(2020, 1, 1, tzinfo=timezone.utc)
        self.date = rd.random.int(min=-100, max=100)

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
    print(type(obj_gene.allele()))
    print(obj_gene.to_json())
#     print(obj_gene.dtype())

# print(result)
# print(result.to_json())
print(rd.Generation.from_json(result.to_json()).value())

# temp = rd.UInt8
# print(f"UInt8 max: {rd.UInt8.max()}")
# print(f"UInt8 min: {rd.UInt8.min()}")
# print(f"UInt8 type: {rd.UInt8}")
# print(f"numpy uint8: {np.uint8()}")

# temp = rd.gene.float(init_range=(-1.0, 1.0), dtype=rd.Float64)
# np_temp = rd.gene.float(init_range=(-1.0, 1.0), dtype=np.float32)

# print(temp)