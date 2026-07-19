# Showcase (excluded from the run-test via the *_showcase.py suffix): these are complete,
# realistic end-to-end examples (full generation counts, ui=True, numba @jit, plotting), so
# they're not run in the fast suite — ruff still lints this file and `mkdocs build --strict`
# validates the `--8<--` includes. Rendered into docs/source/examples.md. Fitness functions
# use `<example>_fitness_fn` names so the shared file has no collisions.

# --8<-- [start:minsum]
import radiate as rd

engine = (
    rd.Engine.int(10, init_range=(0, 100))
    .fitness(lambda x: sum(x))
    .minimizing()
    .select(offspring=rd.Select.elite())
    .alters(rd.Mutate.swap(0.05), rd.Cross.uniform(0.5))
)

result = engine.run(rd.Limit.score(0))

print(result)
# --8<-- [end:minsum]

# --8<-- [start:nqueens]
import numpy as np
import radiate as rd
from numba import jit, uint8

N_QUEENS = 32


@jit(
    uint8(uint8[:]), nopython=True
)  # add this decorator from numba to compile the fitness function to native C code.
def nqueens_fitness_fn(queens: np.ndarray) -> int:
    """Calculate the fitness score for the N-Queens problem."""

    i_indices, j_indices = np.triu_indices(N_QUEENS, k=1)

    same_row = queens[i_indices] == queens[j_indices]

    same_diagonal = np.abs(i_indices - j_indices) == np.abs(
        queens[i_indices] - queens[j_indices]
    )

    return np.sum(same_row) + np.sum(same_diagonal)


engine = (
    rd.Engine.int(N_QUEENS, init_range=(0, N_QUEENS), use_numpy=True, dtype=rd.UInt8)
    .fitness(nqueens_fitness_fn)
    .minimizing()
    .alters(
        rd.Cross.multipoint(0.75, 2),
        rd.Mutate.uniform(0.05),
    )
)

result = engine.run(rd.Limit.score(0), log=False)
print(result)

board = result.value()
for i in range(N_QUEENS):
    for j in range(N_QUEENS):
        if board[j] == i:
            print("Q ", end="")
        else:
            print(". ", end="")
    print()
# --8<-- [end:nqueens]

# --8<-- [start:rastrigin]
import math
import radiate as rd

A = 10.0
RANGE = 5.12
N_GENES = 2


def rastrigin_fitness_fn(x: list[float]) -> float:
    value = A * N_GENES
    for i in range(N_GENES):
        value += x[i] ** 2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
    return value


engine = (
    rd.Engine.float(2, init_range=(-RANGE, RANGE), bounds=(-10.0, 10.0))
    .fitness(rastrigin_fitness_fn)
    .minimizing()
    .alters(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.01))
)

print(engine.run(rd.ScoreLimit(0.0001)))
# --8<-- [end:rastrigin]

# --8<-- [start:dtlz1]
import matplotlib.pyplot as plt
import radiate as rd
import numpy as np
from numba import jit, float32

rd.random.seed(501)

variables = 4
objectives = 3
k = variables - objectives + 1


# Because we are using numpy arrays, we can use numba to compile this function to native code for speed.
# This allows us to match the speed of the rust implementation.
@jit(float32[:](float32[:]), nopython=True)
def dtlz1_fitness_fn(val: np.ndarray) -> np.ndarray:
    g_vals = val[variables - k :] - 0.5
    g = 100.0 * (k + np.sum(g_vals**2 - np.cos(20.0 * np.pi * g_vals)))

    base = 0.5 * (1.0 + g)

    f = np.full(objectives, base, dtype=np.float32)

    for i in range(objectives):
        prod_end = objectives - 1 - i
        if prod_end > 0:
            f[i] *= np.prod(val[:prod_end])

        if i > 0:
            f[i] *= 1.0 - val[objectives - 1 - i]

    return f


engine = (
    rd.Engine.float(variables, use_numpy=True, dtype=rd.Float32)
    .fitness(dtlz1_fitness_fn)
    .objective(rd.MIN, rd.MIN, rd.MIN)
    .front_range(100, 150)
    .select(rd.Select.tournament(k=5), rd.Select.nsga3(points=12))
    .alters(
        rd.Cross.sbx(1.0, 2.0),  # <- Simulated Binary Crossover
        rd.Mutate.uniform(0.1),
    )
)
result = engine.run(rd.GenerationsLimit(2000), ui=True)

# When running an MO problem, we can get the resulting pareto from from the
# engine's epoch result. This is stored in the 'front()' field of the result here:
front = result.front()

fig = plt.figure()
ax = plt.axes(projection="3d")

x = [member.score()[0] for member in front]
y = [member.score()[1] for member in front]
z = [member.score()[2] for member in front]

ax.scatter(x, y, z)
ax.set_xlim((0, 0.5))
ax.set_ylim((0, 0.5))
ax.set_zlim((0, 0.5))
plt.show()
# --8<-- [end:dtlz1]

# --8<-- [start:graph_xor]
import radiate as rd

inputs = [[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]]
answers = [[0.0], [0.0], [1.0], [1.0]]

codec = rd.GraphCodec.directed(
    shape=(2, 1),
    vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
    edge=rd.Op.weight(),
    output=rd.Op.linear(),
)

engine = (
    rd.Engine(codec)
    .regression(inputs, answers, loss=rd.MSE)
    .alters(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1),
    )
)

result = engine.run(rd.Limit.score(0.001), rd.Limit.generations(1000), log=True)

for input, target in zip(inputs, answers):
    print(f"Input: {input}, Target: {target}, Output: {result.value().eval([input])}")
# --8<-- [end:graph_xor]

# --8<-- [start:tree]
import radiate as rd

inputs = [[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]]
answers = [[0.0], [0.0], [1.0], [1.0]]

codec = rd.TreeCodec(
    shape=(2, 1),
    vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.add()],
    root=rd.Op.linear(),
)

engine = (
    rd.Engine(codec)
    .regression(inputs, answers, loss=rd.MSE)
    .alters(rd.Cross.tree(0.7), rd.Mutate.hoist(0.01))
)


result = engine.run(rd.Limit.score(0.01), rd.Limit.seconds(1), log=True)
print(result)

for input, target in zip(inputs, answers):
    print(f"Input: {input}, Target: {target}, Output: {result.value().eval([input])}")
# --8<-- [end:tree]
