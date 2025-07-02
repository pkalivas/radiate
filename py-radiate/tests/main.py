import os
import sys
import math
import matplotlib.pyplot as plt
from numba import jit, cfunc, vectorize
import numpy as np

project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.insert(0, project_root)

import radiate as rd

rd.random.set_seed(501)


class TestHandler(rd.EventHandler):
    def __init__(self):
        super().__init__()
        self.scores = []

    def on_event(self, event):
        
        if event["type"] == "epoch_complete":
            self.scores.append(event["score"])
            # self.scores.append(event['metrics']['unique_members']['value_last'])
        elif event["type"] == "stop":
            plt.plot(self.scores)
            plt.xlabel("Generation")
            plt.ylabel("Best Fitness")
            plt.title("Fitness over Generations")
            plt.show()
        elif event["type"] == "engine_improvement":
            print(f"New best score: {event}")
        


# engine = rd.GeneticEngine(
#     codec=rd.IntCodec.vector(50, (0, 10)),
#     fitness_func=lambda x: sum(x),
#     offspring_selector=rd.BoltzmannSelector(4),
#     objectives="min",
#     # subscribe=TestHandler(),
#     # executor=rd.Executor.WorkerPool(),
#     alters=[
#         rd.MultiPointCrossover(0.75, 2),
#         rd.UniformMutator(0.01)
#     ],
# )

# result = engine.run(rd.ScoreLimit(0), log=True)

# print(result)

inputs = [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]
answers = [[0.0], [1.0], [1.0], [0.0]]


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


def get_dataset():
    inputs = []
    answers = []

    input = -1.0
    for _ in range(-10, 10):
        input += 0.1
        inputs.append([input])
        answers.append([compute(input)])

    return inputs, answers


inputs, answers = get_dataset()

# inputs = [[0.0], [0.0], [0.0], [1.0], [0.0], [0.0], [0.0]]
# answers = [[0.0], [0.0], [1.0], [0.0], [0.0], [0.0], [1.0]]

# def fitness_func(graph: rd.Graph) -> float:
#     error = 0.0
#     outputs = graph.eval(inputs)
#     for output, target in zip(outputs, answers):
#         error += (output[0] - target[0]) ** 2
#     return error / len(answers)

# codec = rd.GraphCodec.directed(
#     shape=(1, 1),
#     vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
#     edge=rd.Op.weight(),
#     output=rd.Op.linear(),
# )

# engine = rd.GeneticEngine(
#     codec=codec,
#     fitness_func=rd.Regression(inputs, answers),
#     objectives="min",
#     alters=[
#         rd.GraphCrossover(0.5, 0.5),
#         rd.OperationMutator(0.07, 0.05),
#         rd.GraphMutator(0.1, 0.1),
#     ],
# )

# result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)], log=True)
# print(result)

# codec = rd.TreeCodec(
#     shape=(1, 1),
#     vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.add()],
#     root=rd.Op.linear(),
# )

# engine = rd.GeneticEngine(
#     codec=codec,
#     fitness_func=rd.Regression(inputs, answers),
#     objectives="min",
#     alters=[
#         rd.TreeCrossover(0.7),
#         rd.HoistMutator(0.01),
#     ],
# )

# result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(100)], log=True)
# print(result)
# print(result.value().__repr__())

# function_inputs = [4.0, -2.0, 3.5, 5.0, -11.0, -4.7]
# desired_output = 44.0

# def genetic_fitness(solution):
#     output = np.sum(np.array(solution) * function_inputs)
#     return np.abs(output - desired_output) + 0.000001


# engine = rd.GeneticEngine(
#     codec=rd.FloatCodec.vector(len(function_inputs), (-4.0, 4.0)),
#     fitness_func=genetic_fitness,
#     objectives="min",
# )

# result = engine.run(rd.ScoreLimit(0.01), log=True)
# print(result)

# #
# print(genetic_fitness(result.value()))

# def encoder():
#     return [1, 'hi', 3.0, True, [1, 2, 3], {'a': 1, 'b': 2}]

# any_codec = rd.AnyCodec(encoder)

# print(any_codec.encode())
# print(any_codec.decode(any_codec.encode()))

# for input, target in zip(inputs, answers): 
#     print(f"Input: {round(input[0], 2)}, Target: {round(target[0], 2)}, Output: {round(result.value().eval([input])[0][0], 2)}")

# print(result.value().eval([[0.5], [0.25], [0.75], [0.1], [0.9]]))

# save the value to json
# with open("best_graph.json", "w") as f:
#     f.write(result.value().to_json())

# for member in result.population():
#     print(member.score())


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
#     # executor=rd.Executor.WorkerPool(),
#     objectives="min",
#     offspring_selector=rd.BoltzmannSelector(4.0),
#     alters=[
#         rd.MultiPointCrossover(0.75, 2),
#         rd.UniformMutator(0.05)
#     ]
# )
# result = engine.run(rd.ScoreLimit(0), log=True)
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

variables = 4
objectives = 3
k = variables - objectives + 1


# @jit(nopython=True, nogil=True)
def dtlz_1(val):
    g = 0.0
    for i in range(variables - k, variables):
        g += (val[i] - 0.5) ** 2 - math.cos(20.0 * math.pi * (val[i] - 0.5))
    g = 100.0 * (k + g)
    f = [0.0] * objectives
    for i in range(objectives):
        f[i] = 0.5 * (1.0 + g)
        for j in range(objectives - 1 - i):
            f[i] *= val[j]
        if i != 0:
            f[i] *= 1.0 - val[objectives - 1 - i]
    return f


# engine = rd.GeneticEngine(
#     codec=rd.FloatCodec.vector(variables, (0.0, 1.0), (-2.0, 2.0)),
#     fitness_func=dtlz_1,
#     offspring_selector=rd.TournamentSelector(k=8),
#     survivor_selector=rd.NSGA2Selector(),
#     objectives=["min" for _ in range(objectives)],
#     alters=[
#         rd.SimulatedBinaryCrossover(1.0, 2.0),
#         rd.UniformMutator(0.1),
#     ],
# )

# result = engine.run(rd.GenerationsLimit(5000), log=True)
# print(result)

# front = result.value()
# print(f"Front size: {len(front)}")
# fig = plt.figure()
# ax = plt.axes(projection="3d")

# x = [member["fitness"][0] for member in front]
# y = [member["fitness"][1] for member in front]
# z = [member["fitness"][2] for member in front]
# ax.scatter(x, y, z, c="r", marker="o")
# ax.set_xlim([0, 0.5])
# ax.set_ylim([0, 0.5])
# ax.set_zlim([0, 0.5])
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
