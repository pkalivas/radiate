#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import radiate as rd
import polars as pl
import matplotlib.pyplot as plt

rd.random.seed(5)


class ScoreDistributionPlotter(rd.EventHandler):
    """
    Subscriber class to handle events and track metrics.
    We will use this to plot score distributions over generations then
    display the plot when the engine stops.
    """

    def __init__(self):
        super().__init__()
        self.history = []

    def on_event(self, event: rd.EngineEvent) -> None:
        if event.event_type() == rd.EventType.EPOCH_COMPLETE:
            ms = event.metrics().to_polars()
            epoch = event.index()
            ms = ms.with_columns(pl.lit(epoch).alias("epoch"))
            self.history.append(ms)
        elif event.event_type() == rd.EventType.STOP:
            df = pl.concat(self.history, how="diagonal_relaxed")
            plot_scores(df)


def plot_scores(ms: pl.DataFrame):
    quant = (
        ms.filter((pl.col("name") == "scores") & (pl.col("kind") == "dist"))
        .select(
            "epoch",
            pl.col("min").alias("q0"),
            pl.col("mean").alias("q50"),
            pl.col("max").alias("q100"),
        )
        .sort("epoch")
    )

    pdf = quant.to_pandas()
    plt.figure(figsize=(8, 5))
    plt.fill_between(
        pdf["epoch"], pdf["q0"], pdf["q100"], alpha=0.2, label="min–max range"
    )
    plt.plot(pdf["epoch"], pdf["q50"], color="C0", linewidth=2, label="mean score")
    plt.xlabel("Epoch")
    plt.ylabel("Score")
    plt.title("Score distribution across generations")
    plt.legend()
    plt.tight_layout()
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
    subscribe=ScoreDistributionPlotter(),
    objective="min",
    alters=[
        rd.GraphCrossover(0.5, 0.5),
        rd.OperationMutator(0.07, 0.05),
        rd.GraphMutator(0.1, 0.1, False),
    ],
)

result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(1000)], log=True)
print(result)
print(result.metrics().dashboard())
