import os
import sys
# import math  # type: ignore
import numpy as np # type: ignore
# from numba import jit, cfunc, vectorize # type: ignore

import matplotlib.pyplot as plt # type: ignore



project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.insert(0, project_root)

import radiate as rd 

print(f"Radiate version: {rd.__version__}")

rd.random.set_seed(500)


# class TestHandler(rd.EventHandler):
#     def __init__(self):
#         super().__init__()
#         self.scores = []

#     def on_event(self, event):
        
#         if event["type"] == "epoch_complete":
#             self.scores.append(event["score"])
#             # self.scores.append(event['metrics']['unique_members']['value_last'])
#         elif event["type"] == "stop":
#             plt.plot(self.scores)
#             plt.xlabel("Generation")
#             plt.ylabel("Best Fitness")
#             plt.title("Fitness over Generations")
#             plt.show()
#         elif event["type"] == "engine_improvement":
#             print(f"New best score: {event}")
        


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
# print(result.metrics()['scores'])

# inputs = [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]
# answers = [[0.0], [1.0], [1.0], [0.0]]


# def compute(x: float) -> float:
#     return 4.0 * x**3 - 3.0 * x**2 + x


# def get_dataset():
#     inputs = []
#     answers = []

#     input = -1.0
#     for _ in range(-10, 10):
#         input += 0.1
#         inputs.append([input])
#         answers.append([compute(input)])

#     return inputs, answers


# inputs, answers = get_dataset()

# # inputs = [[0.0], [0.0], [0.0], [1.0], [0.0], [0.0], [0.0]]
# # answers = [[0.0], [0.0], [1.0], [0.0], [0.0], [0.0], [1.0]]

# # def fitness_func(graph: rd.Graph) -> float:
# #     error = 0.0
# #     outputs = graph.eval(inputs)
# #     for output, target in zip(outputs, answers):
# #         error += (output[0] - target[0]) ** 2
# #     return error / len(answers)

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

# # result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)], log=True)
# # print(result)

# # codec = rd.TreeCodec(
# #     shape=(1, 1),
# #     vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.add()],
# #     root=rd.Op.linear(),
# # )

# # engine = rd.GeneticEngine(
# #     codec=codec,
# #     fitness_func=rd.Regression(inputs, answers),
# #     objectives="min",
# #     alters=[
# #         rd.TreeCrossover(0.7),
# #         rd.HoistMutator(0.01),
# #     ],
# # )

# # result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(100)], log=True)
# # print(result)
# # print(result.value().__repr__())

# # function_inputs = [4.0, -2.0, 3.5, 5.0, -11.0, -4.7]
# # desired_output = 44.0

# # def genetic_fitness(solution):
# #     output = np.sum(np.array(solution) * function_inputs)
# #     return np.abs(output - desired_output) + 0.000001


# # engine = rd.GeneticEngine(
# #     codec=rd.FloatCodec.vector(len(function_inputs), (-4.0, 4.0)),
# #     fitness_func=genetic_fitness,
# #     objectives="min",
# # )

# # result = engine.run(rd.ScoreLimit(0.01), log=True)
# # print(result)

# # #
# # print(genetic_fitness(result.value()))

# # def encoder():
# #     return [1, 'hi', 3.0, True, [1, 2, 3], {'a': 1, 'b': 2}]

# # any_codec = rd.AnyCodec(encoder)

# # print(any_codec.encode())
# # print(any_codec.decode(any_codec.encode()))

# # for input, target in zip(inputs, answers): 
# #     print(f"Input: {round(input[0], 2)}, Target: {round(target[0], 2)}, Output: {round(result.value().eval([input])[0][0], 2)}")

# # print(result.value().eval([[0.5], [0.25], [0.75], [0.1], [0.9]]))

# # save the value to json
# # with open("best_graph.json", "w") as f:
# #     f.write(result.value().to_json())

# # for member in result.population():
# #     print(member.score())


    # score = 0
    # for i in range(N_QUEENS):
    #     for j in range(i + 1, N_QUEENS):
    #         if queens[i] == queens[j]:
    #             score += 1
    #         if abs(i - j) == abs(queens[i] - queens[j]):
    #             score += 1
    # return score


# N_QUEENS = 32

# # @jit(nopython=True, nogil=True)
# def fitness_fn(queens: np.ndarray) -> int:
#     """Calculate the fitness score for the N-Queens problem."""
    
#     i_indices, j_indices = np.triu_indices(N_QUEENS, k=1)
#     same_row = queens[i_indices] == queens[j_indices]
#     same_diagonal = np.abs(i_indices - j_indices) == np.abs(queens[i_indices] - queens[j_indices])
    
#     # Count conflicts
#     score = np.sum(same_row) + np.sum(same_diagonal)
#     return score

# codec = rd.IntCodec.vector(N_QUEENS, (0, N_QUEENS), use_numpy=True)
# engine = rd.GeneticEngine(
#     codec=codec,
#     fitness_func=fitness_fn,
#     objectives="min",
#     offspring_selector=rd.BoltzmannSelector(4.0),
#     alters=[
#         rd.MultiPointCrossover(0.75, 2),
#         rd.UniformMutator(0.05)
#     ]
# )
# result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(1000)], log=True)
# print(result)


# board = result.value()
# for i in range(N_QUEENS):
#     for j in range(N_QUEENS):
#         if board[j] == i:
#             print("Q ", end="")
#         else:
#             print(". ", end="")
#     print()


# # target = "Hello, Radiate!"


# # def fitness_func(x):
# #     return sum(1 for i in range(len(target)) if x[i] == target[i])


# # engine = rd.GeneticEngine(
# #     codec=rd.CharCodec.vector(len(target)),
# #     fitness_func=fitness_func,
# #     diversity=rd.HammingDistance(),
# #     species_threshold=.5,
# #     objectives="max",
# #     offspring_selector=rd.BoltzmannSelector(4),
# # )

# # result = engine.run(rd.ScoreLimit(len(target)))

# # print(result)

# # In your codec or other modules


# # A = 10.0
# # RANGE = 5.12
# # N_GENES = 2

# # def fitness_fn(x):
# #     '''The fitness function for the Rastrigin function.'''
# #     value = A * N_GENES
# #     for i in range(N_GENES):
# #         value += x[i]**2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
# #     return value

# # codec = rd.FloatCodec.vector(2, (-5.12, 5.12))
# # engine = rd.GeneticEngine(codec, fitness_fn)

# # engine.alters([
# #     rd.UniformCrossover(0.5),
# #     rd.ArithmeticMutator(0.01)
# # ])



# # print(engine.run(rd.ScoreLimit(0.0001)))


# variables = 4
# objectives = 3
# k = variables - objectives + 1


# # @jit(nopython=True, nogil=True)
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

# result = engine.run(rd.GenerationsLimit(2000), log=True)
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


# # print()
# # print()
# # gene = rd.Gene.char(allele="a", char_set={"a", "b", "c"})

# # print(gene)


# ########### Test for basic functionality of the library
# ## Weird test - not sure if this functionality should even be enabled

# # float_gene = rd.Gene.float(value_range=(-10.0, 10.0))
# # int_gene = rd.Gene.int(value_range=(0, 10))
# # char_gene = rd.Gene.char(char_set={"a", "b", "c"})
# # bit_gene = rd.Gene.bit()

# # print(float_gene)
# # print(int_gene)
# # print(char_gene)
# # print(bit_gene)

# # float_chrom = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# # int_chrom = rd.Chromosome.int(length=4, value_range=(0, 10))
# # char_chrom = rd.Chromosome.char(length=4, char_set={"a", "b", "c"})
# # bit_chrom = rd.Chromosome.bit(length=4)

# # print(float_chrom)
# # print(int_chrom)
# # print(char_chrom)
# # print(bit_chrom)


# # chrom = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# # chrom2 = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# # chrom3 = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))
# # chrom4 = rd.Chromosome.float(length=4, value_range=(-10.0, 10.0))

# # inputs = [chrom, chrom2, chrom3, chrom4]

# # for i in inputs:
# #     for gene in i.genes():
# #         print(gene)


# # print()
# # math_mutator = rd.SwapMutator(.5)
# # print()

# # mutated = math_mutator.alter([chrom, chrom2, chrom3, chrom4])
# # for gene in mutated.phenotypes():
# #     for chrom in gene.genotype().chromosomes():
# #         for g in chrom.genes():
# #             print(g)


# # # tools/generate_docs.py
# # import ast
# # from pathlib import Path
# # from typing import Dict, List

# # class DocGenerator:
# #     """Generate documentation from examples."""
    
# #     def __init__(self):
# #         self.examples_dir = Path("../examples")  
# #         self.docs_dir = Path("../testsss/examples")
    
# #     def extract_example_info(self, filepath: Path) -> Dict[str, str]:
# #         """Extract information from example file."""
# #         try:
# #             with open(filepath, 'r') as f:
# #                 content = f.read()
            
# #             # Parse the file
# #             tree = ast.parse(content)
            
# #             # Extract module docstring
# #             docstring = ast.get_docstring(tree) or ""
            
# #             # Look for main function
# #             main_func = None
# #             for node in ast.walk(tree):
# #                 if isinstance(node, ast.FunctionDef) and node.name == 'main':
# #                     main_func = ast.get_docstring(node) or ""
# #                     break
            
# #             # Get category from directory structure
# #             category = filepath.parent.name
            
# #             return {
# #                 'name': filepath.stem.replace('_', ' ').title(),
# #                 'description': docstring,
# #                 'main_description': main_func,
# #                 'filename': filepath.name,
# #                 'category': category,
# #                 'full_path': str(filepath.relative_to(self.examples_dir))
# #             }
# #         except Exception as e:
# #             print(f"Warning: Could not parse {filepath}: {e}")
# #             return {
# #                 'name': filepath.stem.replace('_', ' ').title(),
# #                 'description': f"Example: {filepath.name}",
# #                 'main_description': "",
# #                 'filename': filepath.name,
# #                 'category': filepath.parent.name,
# #                 'full_path': str(filepath.relative_to(self.examples_dir))
# #             }
    
# #     def generate_example_docs(self):
# #         """Generate documentation for all examples."""
# #         self.docs_dir.mkdir(parents=True, exist_ok=True)
        
# #         examples = []
        
# #         # Find all Python files in examples directory
# #         for filepath in self.examples_dir.rglob("*.py"):
# #             if filepath.name != "__init__.py" and "test_" not in filepath.name:
# #                 info = self.extract_example_info(filepath)
# #                 examples.append(info)
        
# #         # Group by category
# #         categories = {}
# #         for example in examples:
# #             cat = example['category']
# #             if cat not in categories:
# #                 categories[cat] = []
# #             categories[cat].append(example)
        
# #         # Generate index
# #         with open(self.docs_dir / "index.md", 'w') as f:
# #             f.write("# Radiate Examples\n\n")
# #             f.write("This directory contains examples demonstrating Radiate's capabilities.\n\n")
            
# #             # Table of contents
# #             f.write("## Table of Contents\n\n")
# #             for category in sorted(categories.keys()):
# #                 f.write(f"- [{category.title()}](#{category.lower()})\n")
# #             f.write("\n")
            
# #             # Examples by category
# #             for category in sorted(categories.keys()):
# #                 f.write(f"## {category.title()}\n\n")
                
# #                 for example in sorted(categories[category], key=lambda x: x['name']):
# #                     f.write(f"### {example['name']}\n\n")
                    
# #                     if example['description']:
# #                         f.write(f"{example['description']}\n\n")
                    
# #                     f.write(f"**File:** `{example['full_path']}`\n\n")
                    
# #                     if example['main_description']:
# #                         f.write(f"{example['main_description']}\n\n")
                    
# #                     f.write("---\n\n")
        
# #         print(f"Generated documentation for {len(examples)} examples in {len(categories)} categories")
        
# #         # Also generate individual files for each category
# #         for category, category_examples in categories.items():
# #             category_file = self.docs_dir / f"{category}.md"
# #             with open(category_file, 'w') as f:
# #                 f.write(f"# {category.title()} Examples\n\n")
                
# #                 for example in sorted(category_examples, key=lambda x: x['name']):
# #                     f.write(f"## {example['name']}\n\n")
                    
# #                     if example['description']:
# #                         f.write(f"{example['description']}\n\n")
                    
# #                     f.write(f"**File:** `{example['full_path']}`\n\n")
                    
# #                     if example['main_description']:
# #                         f.write(f"{example['main_description']}\n\n")
                    
# #                     # Try to extract code snippets
# #                     try:
# #                         code_snippet = self.extract_code_snippet(example['full_path'])
# #                         if code_snippet:
# #                             f.write("### Code Snippet\n\n")
# #                             f.write("```python\n")
# #                             f.write(code_snippet)
# #                             f.write("\n```\n\n")
# #                     except:
# #                         pass
                    
# #                     f.write("---\n\n")
    
# #     def extract_code_snippet(self, filepath: str) -> str:
# #         """Extract a code snippet from the example file."""
# #         full_path = self.examples_dir / filepath
# #         try:
# #             with open(full_path, 'r') as f:
# #                 lines = f.readlines()
            
# #             # Find the main function or first code block
# #             start_line = 0
# #             for i, line in enumerate(lines):
# #                 if line.strip().startswith('def main(') or line.strip().startswith('if __name__'):
# #                     start_line = i
# #                     break
            
# #             # Extract up to 20 lines
# #             snippet_lines = lines[start_line:start_line + 20]
# #             return ''.join(snippet_lines).strip()
# #         except:
# #             return ""

# # if __name__ == "__main__":
# #     generator = DocGenerator()
# #     generator.generate_example_docs()



