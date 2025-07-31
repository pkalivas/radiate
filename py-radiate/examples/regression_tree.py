#!/usr/bin/env python3
"""
Regression with Tree Codec

This example demonstrates using the TreeCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import radiate as rd
import matplotlib.pyplot as plt  # type: ignore
import networkx as nx  # type: ignore

rd.random.set_seed(500)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])

engine = rd.GeneticEngine(
    codec=rd.TreeCodec(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.add()],
        root=rd.Op.linear(),
    ),
    fitness_func=rd.Regression(inputs, answers),
    objectives="min",
    alters=[
        rd.TreeCrossover(0.7),
        rd.HoistMutator(0.01),
    ],
)


result = engine.run([rd.ScoreLimit(0.01), rd.GenerationsLimit(1000)], log=True)
print(result)

best_tree = result.value()

G = nx.DiGraph()


def plot_tree(tree, ax, x=0, y=0, width=1.0, depth=0):
    """Recursively plot tree structure"""
    op_name = tree.value()

    ax.plot(
        x,
        y,
        "o",
        markersize=20,
        color="lightblue",
        markeredgecolor="black",
        markeredgewidth=2,
    )
    ax.text(
        x,
        y,
        op_name,
        ha="center",
        va="center",
        fontsize=8,
        fontweight="bold",
        color="black",
    )

    if tree.children():
        num_children = len(tree.children())
        child_width = width / num_children

        for i, child in enumerate(tree.children()):
            child_x = x - width / 2 + child_width / 2 + i * child_width
            child_y = y - 1

            ax.plot([x, child_x], [y, child_y], "k-", linewidth=2)
            plot_tree(child, ax, child_x, child_y, child_width, depth + 1)


plt.figure(figsize=(12, 8))
ax = plt.gca()
ax.set_title("Evolved Tree Structure")
ax.axis("off")

plot_tree(best_tree.nodes()[0], ax, x=0, y=0, width=10.0)

plt.show()
