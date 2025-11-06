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


class Subscriber(rd.EventHandler):
    def __init__(self):
        super().__init__()
        self.history = []

    def on_event(self, event):
        if event.event_type() == rd.EventType.EPOCH_COMPLETE:
            ms = event.metrics().to_polars()
            epoch = event.index()

            # Add epoch column so we can track progress over time
            ms = ms.with_columns(pl.lit(epoch).alias("epoch"))
            self.history.append(ms)
        elif event.event_type() == rd.EventType.STOP:
            self.on_stop(event)

    def on_stop(self, event):
        """Optionally handle STOP event to plot automatically."""
        df = self.get_full_metrics()
        plot_scores(df)

    def get_full_metrics(self):
        return pl.concat(self.history, how="diagonal_relaxed")


def plot_scores(ms: pl.DataFrame):
    # Extract all "scores" distributions across epochs
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
    subscribe=Subscriber(),
    objectives="min",
    alters=[
        rd.GraphCrossover(0.5, 0.5),
        rd.OperationMutator(0.07, 0.05),
        rd.GraphMutator(0.1, 0.1, False),
    ],
)

result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(1000)], log=True)
print(result)
print(result.metrics().dashboard())
