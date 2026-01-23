#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import radiate as rd
import polars as pl  # type: ignore
import numpy as np
import matplotlib.pyplot as plt

rd.random.seed(567123)


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
            df = pl.DataFrame(
                {"Generation": list(range(len(self.scores))), "Score": self.scores}
            )
            plt.plot(df["Generation"], df["Score"])
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

engine = rd.GeneticEngine(
    codec=rd.GraphCodec.directed(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    ),
    fitness_func=rd.Regression(inputs, answers, batch=True),
    subscribe=ScorePlotterHandler(),
    objective="min",
    alters=[
        rd.GraphCrossover(rd.Rate.fixed(0.05), 0.5),
        rd.OperationMutator(0.07, 0.05),
        rd.GraphMutator(0.1, 0.1, False),
    ],
)

result = engine.run(
    [rd.ScoreLimit(0.001), rd.GenerationsLimit(1000)],
    log=True,
)

eval_results = result.value().eval(inputs)
accuracy = np.mean(
    np.abs(np.array(eval_results).flatten() - np.array(answers).flatten()) < 0.1
)

print(result)
print(result.metrics().dashboard())
print(f"Accuracy within 0.1: {accuracy * 100:.2f}%")
