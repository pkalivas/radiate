#!/usr/bin/env python3
# /// script
# requires-python = ">=3.13"
# dependencies = [
#   "numpy",
#   "numba",
# ]
# ///
"""
N-Queens Problem with Radiate
This example demonstrates solving the N-Queens problem using Radiate's genetic algorithm capabilities.
The N-Queens problem is a classic combinatorial problem where the goal is to place N queens
on an N x N chessboard such that no two queens threaten each other.
This means that no two queens can be in the same row, column, or diagonal.
"""

# pyright: reportMissingImports=false

import os
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import numpy as np
import radiate as rd
from numba import jit, uint8

rd.random.seed(514)

N_QUEENS = 40


@jit(uint8(uint8[:]), nopython=True)
def fit(queens: np.ndarray) -> int:
    """Calculate the fitness score for the N-Queens problem."""

    i_indices, j_indices = np.triu_indices(N_QUEENS, k=1)

    same_row = queens[i_indices] == queens[j_indices]

    same_diagonal = np.abs(i_indices - j_indices) == np.abs(
        queens[i_indices] - queens[j_indices]
    )

    return np.sum(same_row) + np.sum(same_diagonal)


engine = (
    rd.Engine.int(N_QUEENS, init_range=(0, N_QUEENS), use_numpy=True, dtype=rd.UInt8)
    .fitness(fit)
    .minimizing()
    .alters(
        rd.Cross.multipoint(0.75, 2),
        rd.Mutate.uniform(0.05),
    )
)


result = engine.run(rd.Limit.score(0), ui=True)
print(result)


board = result.value()
for i in range(N_QUEENS):
    for j in range(N_QUEENS):
        if board[j] == i:
            print("Q ", end="")
        else:
            print(". ", end="")
    print()
