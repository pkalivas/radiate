#!/usr/bin/env python3
"""
Example demonstrating the new wrapper decorator system for Python-Rust bridge objects.
"""

import radiate as rd


# Example 1: Using the convenience decorators
print("=== Example 1: Using convenience decorators ===")

codec = rd.FloatCodec(
    [
        0.0, 1.0, 2.0, 3.0, 4.0, 5.0
    ]
)

print(codec.encode())

# # Create a population using the new wrapper system
# phenotypes = [
#     rd.Phenotype(
#         genotype=rd.Genotype([
#             rd.Chromosome.float(length=3, value_range=(0.0, 10.0))
#         ]),
#         score=0.5
#     ),
#     rd.Phenotype(
#         genotype=rd.Genotype([
#             rd.Chromosome.float(length=3, value_range=(0.0, 10.0))
#         ]),
#         score=0.8
#     )
# ]

# population = rd.Population(phenotypes)
# print(f"Population length: {len(population)}")
# print(f"Population gene type: {population.gene_type()}")

# # Iterate over phenotypes
# for i, phenotype in enumerate(population):
#     print(f"Phenotype {i}: {phenotype}")
#     print(f"  Score: {phenotype.score()}")
#     print(f"  Gene type: {phenotype.gene_type()}")

# py = population.to_python()
# print(f"Converted to Python object: {py}")
# other = rd.Population.from_python(py)
# print(f"Converted back to Population: {other}")
