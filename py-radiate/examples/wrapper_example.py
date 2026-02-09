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
        super().__init__()
        self.te=    rd.FieldSpec.scalar(name="lr", dtype=rd.Float64, init_range=(1e-5, 1e-1), bounds=(1e-5, 1e-1)),
        self.number = rd.random.int(min=0, max=10)
        self.date = [rd.random.int(min=1, max=28) for _ in range(3)]
        self.o = True

    def __repr__(self):
        return f"ObjectGene(number={self.number}, date={self.date})"


@rd.fitness(batch=True)
def fitness_function(phenotypes: list[list[ObjectGene]]) -> list[float]:
    return [sum(gene.number for gene in individual) for individual in phenotypes]


engine = rd.GeneticEngine(
    rd.AnyCodec(ObjectGene() for _ in range(10)),
    fitness_func=fitness_function,
    objective="min",
)

result = engine.run(rd.ScoreLimit(0), log=True)

# codec = rd.AnyCodec(ObjectGene() for _ in range(10))
# print(codec.encode().dtype())
# print(result.population().dtype())

# temp = ObjectGene()
# print(temp.dtype())

# print(result.value())
# print(rd.Generation.from_json(result.to_json()).value())
# print(result.ecosystem().dtype())

# import radiate as rd
# from radiate.field import FieldSpec, FieldCodec

specs = [
    rd.FieldSpec.scalar(name="lr", dtype=rd.Float64, init_range=(1e-5, 1e-1), bounds=(1e-5, 1e-1)),
    rd.FieldSpec.scalar(name="depth", dtype=rd.Int64, init_range=(1, 12), bounds=(1, 12)),
    rd.FieldSpec.scalar(name="use_bias", dtype=rd.Boolean, init_range=(0, 1), bounds=(0, 1)),
    rd.FieldSpec.list(
        len=4,
        inner=rd.FieldSpec.scalar(name="layers", dtype=rd.Int64, init_range=(8, 256), bounds=(8, 256)),
    ),
    rd.FieldSpec.scalar(
        name="activation",
        dtype=rd.String,
        choices=["relu", "tanh", "sigmoid"],
    ),
]

codec = rd.FieldCodec(count=1, specs=specs)
gt = codec.encode()
# print(gt)
phenotype = codec.decode(gt)
print(phenotype)  # list[dict-like AnyValue structs], length 10

# print()
# print(result.to_json())
# print()


# for obj_gene in result.value():
#     print(type(obj_gene.allele()))
    # print(obj_gene.to_json())
#     print(obj_gene.dtype())

# print(result)
# print(result.to_json())
# print(rd.Generation.from_json(result.to_json()).value())

# temp = rd.UInt8
# print(f"UInt8 max: {rd.UInt8.max()}")
# print(f"UInt8 min: {rd.UInt8.min()}")
# print(f"UInt8 type: {rd.UInt8}")
# print(f"numpy uint8: {np.uint8()}")

# temp = rd.gene.float(init_range=(-1.0, 1.0), dtype=rd.Float64)
# np_temp = rd.gene.float(init_range=(-1.0, 1.0), dtype=np.float32)

# print(temp)
# print(np_temp)
