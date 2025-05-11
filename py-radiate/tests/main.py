import os
import sys
import math

project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, project_root)


import radiate as rd

rd.random.set_seed(100)

# target = "Hello, Radiate!"
# def fitness_fn(x):
#     '''The fitness function for the string matching problem.'''
#     x = x[0]
#     value = 0
#     for i in range(len(x)):
#         if x[i] == target[i]:
#             value += 1
#     return value

# codex = rd.CharCodex([len(target)])
# engine = rd.GeneticEngine(codex, fitness_fn)
# engine.maximizing()
# engine.offspring_selector(rd.BoltzmannSelector(4))

# result = engine.run(rd.ScoreLimit(len(target)), log=False)

# print(result)

codex = rd.IntCodex([10], (0, 10))
engine = rd.GeneticEngine(codex, lambda x: sum(x[0]))
engine.offspring_selector(rd.BoltzmannSelector(4))
engine.alters([
    rd.MultiPointCrossover(0.75, 2), 
    rd.UniformMutator(0.01)
])

result = engine.run(rd.ScoreLimit(0), log=False)

print(result)

# A = 10.0
# RANGE = 5.12
# N_GENES = 2

# def fitness_fn(x):
#     '''The fitness function for the Rastrigin function.'''
#     x = x[0]
#     value = A * N_GENES
#     for i in range(N_GENES):
#         value += x[i]**2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
#     return value

# codex = rd.FloatCodex([2], (-5.12, 5.12))
# engine = rd.Engine(codex, fitness_fn)

# engine.alters([
#     rd.UniformCrossover(0.5), 
#     rd.ArithmeticMutator(0.01)
# ])

# engine.run(rd.ScoreLimit(0.0001))




# import inspect
# import ast

# def test_fn(*args):
#     return args

# bound = inspect.signature(test_fn).bind([1, 2, 3])
# print(bound)
# print(bound.arguments) 

# def fitness(x):
#     return sum(x)

# def get_expr_body(fn):
#     tree = ast.parse(inspect.getsource(fn))
#     fn_node = tree.body[0]

#     if isinstance(fn_node, ast.FunctionDef):
#         body = fn_node.body[0]
#         if isinstance(body, ast.Return):
#             return body.value  # the actual expression returned
#     raise ValueError("Unsupported function format")

# class Expr:
#     def __init__(self, op, args):
#         self.op = op
#         self.args = args

#     def __repr__(self):
#         return f"Expr({self.op}, {self.args})"

# def ast_to_expr(node):
#     if isinstance(node, ast.Call):
#         func_name = node.func.id  # assuming no nested calls
#         args = [ast_to_expr(arg) for arg in node.args]
#         return Expr(func_name, args)
#     elif isinstance(node, ast.Name):
#         return node.id
#     elif isinstance(node, ast.Constant):
#         return node.value
#     else:
#         raise NotImplementedError(f"Unsupported node: {ast.dump(node)}")

# source = inspect.getsource(fitness)
# tree = ast.parse(source)
# print(ast.dump(tree, indent=4))

# expr_node = get_expr_body(fitness)
# expr = ast_to_expr(expr_node)
# print(expr)  


# for name, param in bound.parameters.items():
#     print(f"{name=}, {param.kind=}, {param.default=}")