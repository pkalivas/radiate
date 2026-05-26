# --8<-- [start:nqueens]
import numpy as np
import radiate as rd
from numba import jit, uint8

rd.random.seed(514)

N_QUEENS = 32


@jit(uint8(uint8[:]), nopython=True)
def fitness_fn(queens: np.ndarray) -> int:
    """Calculate the fitness score for the N-Queens problem."""

    i_indices, j_indices = np.triu_indices(N_QUEENS, k=1)

    same_row = queens[i_indices] == queens[j_indices]

    same_diagonal = np.abs(i_indices - j_indices) == np.abs(
        queens[i_indices] - queens[j_indices]
    )

    return np.sum(same_row) + np.sum(same_diagonal)


engine = (
    rd.Engine.int(
        N_QUEENS, init_range=(0, N_QUEENS), use_numpy=True, dtype=rd.UInt8
    )  # <- note the UInt8 dtype here, which matches the numba function's expected input
    .fitness(fitness_fn)
    .minimizing()
    .limit(rd.Limit.score(0))
    .alters(
        rd.Cross.multipoint(0.75, 2),
        rd.Mutate.uniform(0.05),
    )
)


result = engine.run(ui=True)
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
