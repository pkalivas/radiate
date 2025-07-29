#!/usr/bin/env python3
"""
N-Queens Problem with Radiate
This example demonstrates solving the N-Queens problem using Radiate's genetic algorithm capabilities.
The N-Queens problem is a classic combinatorial problem where the goal is to place N queens
on an N x N chessboard such that no two queens threaten each other.
This means that no two queens can be in the same row, column, or diagonal.
"""

import numpy as np
import radiate as rd

N_QUEENS = 32

def fitness_fn(queens: np.ndarray) -> int:
    """Calculate the fitness score for the N-Queens problem."""
    
    i_indices, j_indices = np.triu_indices(N_QUEENS, k=1)
    same_row = queens[i_indices] == queens[j_indices]
    same_diagonal = np.abs(i_indices - j_indices) == np.abs(queens[i_indices] - queens[j_indices])
    
    # Count conflicts
    score = np.sum(same_row) + np.sum(same_diagonal)
    return score

codec = rd.IntCodec.vector(N_QUEENS, (0, N_QUEENS), use_numpy=True)
engine = rd.GeneticEngine(
    codec=codec,
    fitness_func=fitness_fn,
    objectives="min",
    offspring_selector=rd.BoltzmannSelector(4.0),
    alters=[
        rd.MultiPointCrossover(0.75, 2),
        rd.UniformMutator(0.05)
    ]
)
result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(1000)], log=True)
print(result)


board = result.value()
for i in range(N_QUEENS):
    for j in range(N_QUEENS):
        if board[j] == i:
            print("Q ", end="")
        else:
            print(". ", end="")
    print()
