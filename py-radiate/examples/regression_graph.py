#!/usr/bin/env python3
# /// script
# requires-python = ">=3.13"
# dependencies = [
#   "matplotlib",
# ]
# ///
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

# pyright: reportMissingImports=false

import os
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import radiate as rd
import matplotlib.pyplot as plt  # type: ignore

rd.random.seed(67123)


class ScorePlotterHandler(rd.EventHandler):
    """
    Subscriber class to handle events and track metrics.
    We will use this to plot score distributions over generations then
    display the plot when the engine stops.
    """

    def __init__(self):
        super().__init__()
        self.scores = []

    def on_event(self, event: rd.EngineEvent) -> None:
        if event.event_type() == rd.EventType.EPOCH_COMPLETE:
            best_score = event.score()
            self.scores.append(best_score)
        elif event.event_type() == rd.EventType.STOP:
            plt.plot(list(range(len(self.scores))), self.scores)
            plt.xlabel("Generation")
            plt.ylabel("Best Score")
            plt.title("Best Score over Generations")
            plt.grid(True)
            plt.show()


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])

engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    .regression(inputs, answers, loss=rd.MSE)
    .subscribe(ScorePlotterHandler())
    .alters(
        rd.Cross.graph(0.05, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.generations(1000), rd.Limit.score(0.001))
)

result = engine.run(log=True)

eval_results = result.value().eval(inputs)
accuracy = rd.accuracy(result.value(), inputs, answers, loss=rd.MSE)

print(result)
print(result.metrics().dashboard())
print(accuracy)
