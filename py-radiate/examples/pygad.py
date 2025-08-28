#!/usr/bin/env python3
"""
Example of using radiate to solve the PyGAD example problem found on their homepage.

https://pygad.readthedocs.io/en/latest/

This example should resolve within ~15-20 generations, or about 1ms
depending on your random seed & your machine.
"""

import numpy as np
import radiate as rd
from numba import float32, jit

rd.random.seed(42)

function_inputs = np.array([4.0, -2.0, 3.5, 5.0, -11.0, -4.7])
desired_output = 44.0


class MutateTemp(rd.Mutator):
    def __init__(self, mutation_rate: float) -> None:
        super().__init__(mutation_rate)
        self.mutation_rate = mutation_rate

    def mutate(self, chromosome: rd.Chromosome) -> rd.Chromosome:
        for i in range(len(chromosome)):
            if rd.random.randfloat() < self.mutation_rate:
                chromosome[i] = chromosome.view(i).new_instance()
        return chromosome


class CrossoverTemp(rd.Crossover):
    def __init__(self, crossover_rate: float) -> None:
        super().__init__(rate=crossover_rate)
        self.crossover_rate = crossover_rate

    def crossover(
        self, parent1: rd.Chromosome, parent2: rd.Chromosome
    ) -> tuple[rd.Chromosome, rd.Chromosome]:
        for i in range(len(parent1)):
            if rd.random.randfloat() < self.crossover_rate:
                parent1[i], parent2[i] = parent2.view(i), parent1.view(i)
        return parent1, parent2


@jit(float32(float32[:]), nopython=True)
def fitness(solution: np.ndarray) -> float:
    output = np.sum(solution * function_inputs)
    return np.abs(output - desired_output)


engine = rd.GeneticEngine(
    codec=rd.FloatCodec.vector(len(function_inputs), (-4.0, 4.0), use_numpy=True),
    fitness_func=fitness,
    objectives="min",
    alters=[
        CrossoverTemp(0.5),
        MutateTemp(0.1),
    ],
)

result = engine.run(rd.ScoreLimit(0.01), log=True)

print(f"\nBest solution found: {result.value()}")
print(f"Fitness: {result.score()}")
print(f"Generations completed: {result.index()}")
print(f"Function output: {np.sum(result.value() * function_inputs)}")

print(result)
