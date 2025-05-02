import os
import sys

# Add the project root to Python path
project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, project_root)

print(project_root)


import radiate as rd

codec = rd.Genome(
    gene_type='float',
    num_genes=5,
    num_chromosomes=1,
    min_value=0.0,
    max_value=1.0,
    min_bound=0.0,
    max_bound=1.0,
)
# print(codec.encode())
# print(codec.decode([0.1, 0.2, 0.3, 0.4, 0.5]))

encoder = lambda: rd.AnyChromosome(["a", "b", "c", "d", "e", 1, 2, 3, 4, 5])


import inspect
import ast

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

def fitness_fn(x):
    """Fitness function to maximize the sum of the input list."""
    return sum(x[0])


engine = rd.Engine(
    genome=rd.Genome(
        gene_type='float',
        num_genes=100,
        num_chromosomes=1,
        min_value=0.0,
        max_value=1.0,
        min_bound=0.0,
        max_bound=1.0,
    ),
    problem=rd.Problem(
        fitness_fn=fitness_fn
    ),
)

engine.run(num_generations=100)

# print(engine.encode())

# any_genes = rd.AnyGene(allele=12333)

# print(any_genes.allele())


# float = Gene(
    
#     min_value=0.0,
#     max_value=1.0,
# )

# int = Gene(
    
#     min_value=0,
#     max_value=10,
# )
# bool = Gene(
#     type='bit',
# )
# char = Gene(
    
#     allele='a',
# )
# any = Gene(
    
#     allele=12333,
# )

# print(repr(float))
# print(int)
# print(bool)
# print(char)
# print(any)


# chromosome = Chromosome(
#     num_genes=5,
#     type='float',
#     min_value=0.0,
#     max_value=1.0,
# )

# print(chromosome)

# for gene in chromosome:
#     print(gene)
    