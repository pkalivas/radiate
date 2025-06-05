import os
import sys
import math
import matplotlib.pyplot as plt

project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.insert(0, project_root)

import radiate as rd
from numba import jit, cfunc, vectorize


rd.random.set_seed(501)

engine = rd.GeneticEngine(
    codec=rd.IntCodec.vector(10, (0, 10)),
    fitness_func=lambda x: sum(x),
    offspring_selector=rd.BoltzmannSelector(4),
    alters=[
        rd.MultiPointCrossover(0.75, 2), 
        rd.UniformMutator(0.01)
    ],
)

result = engine.run(rd.ScoreLimit(0))

print(result)


# N_QUEENS = 32

# @jit(nopython=True, nogil=True)
# def fitness_fn(queens):
#     """Calculate the fitness score for the N-Queens problem."""
#     score = 0
#     for i in range(N_QUEENS):
#         for j in range(i + 1, N_QUEENS):
#             if queens[i] == queens[j]:
#                 score += 1
#             if abs(i - j) == abs(queens[i] - queens[j]):
#                 score += 1
#     return score

# codec = rd.IntCodec.vector(N_QUEENS, (0, N_QUEENS ))
# engine = rd.GeneticEngine(
#     codec=codec,
#     fitness_func=fitness_fn,
#     num_threads=1,
#     offspring_selector=rd.BoltzmannSelector(4.0),
#     alters=[
#         rd.MultiPointCrossover(0.75, 2),
#         rd.UniformMutator(0.05)
#     ]
# )
# result = engine.run(rd.ScoreLimit(0), log=False)
# print(result)

# board = result.value()
# for i in range(N_QUEENS):
#     for j in range(N_QUEENS):
#         if board[j] == i:
#             print("Q ", end="")
#         else:
#             print(". ", end="")
#     print()


# target = "Hello, Radiate!"


# def fitness_func(x):
#     return sum(1 for i in range(len(target)) if x[i] == target[i])


# engine = rd.GeneticEngine(
#     codec=rd.CharCodec.vector(len(target)),
#     fitness_func=fitness_func,
#     diversity=rd.HammingDistance(),
#     species_threshold=.5,
#     objectives="max",
#     offspring_selector=rd.BoltzmannSelector(4),
# )

# result = engine.run(rd.ScoreLimit(len(target)))

# print(result)

# A = 10.0
# RANGE = 5.12
# N_GENES = 2

# def fitness_fn(x):
#     '''The fitness function for the Rastrigin function.'''
#     value = A * N_GENES
#     for i in range(N_GENES):
#         value += x[i]**2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
#     return value

# codec = rd.FloatCodec.vector(2, (-5.12, 5.12))
# engine = rd.GeneticEngine(codec, fitness_fn)

# engine.alters([
#     rd.UniformCrossover(0.5),
#     rd.ArithmeticMutator(0.01)
# ])

# print(engine.run(rd.ScoreLimit(0.0001)))

# variables = 4
# objectives = 3
# k = variables - objectives + 1


# @jit(nopython=True, nogil=True)
# def dtlz_1(val):
#     g = 0.0
#     for i in range(variables - k, variables):
#         g += (val[i] - 0.5) ** 2 - math.cos(20.0 * math.pi * (val[i] - 0.5))
#     g = 100.0 * (k + g)
#     f = [0.0] * objectives
#     for i in range(objectives):
#         f[i] = 0.5 * (1.0 + g)
#         for j in range(objectives - 1 - i):
#             f[i] *= val[j]
#         if i != 0:
#             f[i] *= 1.0 - val[objectives - 1 - i]
#     return f


# engine = rd.GeneticEngine(
#     codec=rd.FloatCodec.vector(variables, (0.0, 1.0), (-100.0, 100.0)),
#     fitness_func=dtlz_1,
#     offspring_selector=rd.TournamentSelector(k=5),
#     survivor_selector=rd.NSGA2Selector(),
#     objectives=["min" for _ in range(objectives)],
#     alters=[
#         rd.SimulatedBinaryCrossover(1.0, 1.0),
#         rd.UniformMutator(0.1)
#     ],
# )

# result = engine.run(rd.GenerationsLimit(1000))
# print(result)

# front = result.value()
# fig = plt.figure()
# ax = plt.axes(projection="3d")

# x = [member["fitness"][0] for member in front]
# y = [member["fitness"][1] for member in front]
# z = [member["fitness"][2] for member in front]
# ax.scatter(x, y, z, c="r", marker="o")
# plt.show()


# print()
# print()
# gene = rd.Gene.char(allele="a", char_set={"a", "b", "c"})

# print(gene)


########### Test for basic functionality of the library
## Weird test - not sure if this functionality should even be enabled

# float_gene = rd.Gene.float(value_range=(-10.0, 10.0))
# int_gene = rd.Gene.int(value_range=(0, 10))
# char_gene = rd.Gene.char(char_set={"a", "b", "c"})
# bit_gene = rd.Gene.bit()

# print(float_gene)
# print(int_gene)
# print(char_gene)
# print(bit_gene)

# float_chrom = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# int_chrom = rd.Chromosome.int(length=4, value_range=(0, 10))
# char_chrom = rd.Chromosome.char(length=4, char_set={"a", "b", "c"})
# bit_chrom = rd.Chromosome.bit(length=4)

# print(float_chrom)
# print(int_chrom)
# print(char_chrom)
# print(bit_chrom)


# chrom = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# chrom2 = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# chrom3 = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# chrom4 = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))

# inputs = [chrom, chrom2, chrom3, chrom4]

# for i in inputs:
#     for gene in i.genes():
#         print(gene)


# print()
# math_mutator = rd.SwapMutator(.5)
# print()

# mutated = math_mutator.alter([chrom, chrom2, chrom3, chrom4])
# for gene in mutated.phenotypes():
#     for chrom in gene.genotype().chromosomes():
#         for g in chrom.genes():
#             print(g)
