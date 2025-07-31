#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import radiate as rd
# import matplotlib.pyplot as plt  # type: ignore
# import networkx as nx  # type: ignore


rd.random.set_seed(518)


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
    codec=rd.GraphCodec.directed(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    ),
    fitness_func=rd.Regression(inputs, answers),
    objectives="min",
    alters=[
        rd.GraphCrossover(0.5, 0.5),
        rd.OperationMutator(0.07, 0.05),
        rd.GraphMutator(0.1, 0.1),
    ],
)

result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(1000)], log=True)
print(result)

# import numpy as np

# print(np.array(result.population().py_population()))


# graph = result.value()
# nodes = graph.nodes()

# G = nx.DiGraph()

# for i, node in enumerate(nodes):
#     node_type = node.node_type()
#     op_name = node.value()
#     G.add_node(node.index(), label=f"{node_type}: {op_name}")

# # Add edges based on the graph connections
# for i, node in enumerate(nodes):
#     for outgoing in node.outgoing():
#         G.add_edge(node.index(), outgoing)

# plt.figure(figsize=(12, 8))
# pos = nx.spring_layout(G, k=3, iterations=50)

# node_colors = []
# for node in G.nodes():
#     node_type = G.nodes[node]["label"].split(":")[0]
#     if "input" in node_type:
#         node_colors.append("lightblue")
#     elif "output" in node_type:
#         node_colors.append("lightgreen")
#     elif "vertex" in node_type:
#         node_colors.append("lightcoral")
#     elif "edge" in node_type:
#         node_colors.append("lightyellow")
#     else:
#         node_colors.append("lightgray")

# nx.draw(
#     G,
#     pos,
#     node_color=node_colors,
#     node_size=2000,
#     font_size=8,
#     font_weight="bold",
#     arrows=True,
#     edge_color="gray",
#     arrowsize=20,
# )

# labels = {node: G.nodes[node]["label"] for node in G.nodes()}
# nx.draw_networkx_labels(G, pos, labels)

# plt.title("Evolved Graph Network Structure")
# plt.axis("off")
# plt.tight_layout()
# plt.show()

# # Print graph information
# print("\nGraph Network Information:")




# # codec = rd.GraphCodec.directed(
# #     shape=(1, 1),
# #     vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
# #     edge=rd.Op.weight(),
# #     output=rd.Op.linear(),
# # )
# # novelty_engine = rd.GeneticEngine(
# #     codec=codec,
# #     fitness_func=rd.NoveltySearch(
# #         distance=rd.GraphTopologyDistance(),
# #         k=10,
# #         threshold=0.01,
# #         archive_size=1000,
# #     ),
# #     executor=rd.Executor.WorkerPool(),
# #     alters=[
# #         rd.GraphCrossover(0.5, 0.5),
# #         rd.OperationMutator(0.1, 0.1),
# #         rd.GraphMutator(0.08, 0.10, False),
# #     ],
# # )

# # novelty_result = novelty_engine.run(
# #     [rd.GenerationsLimit(500)],
# #     log=True,
# # )

# # print("Novelty Search Result:")
# # print(f"Generations: {novelty_result}")

# # graph = codec.decode(novelty_result.population()[0].genotype())
