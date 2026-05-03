#!/usr/bin/env python3
"""
Regression with Graph Codec

This example demonstrates using the GraphCodec to solve a regression problem.
We have a simple polynomial function and we want to evolve a graph that approximates it.
"""

import radiate as rd

rd.random.seed(67123)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])


target_species = 4.0
rolling = int(target_species)

spec_count_signal = rd.select("count.species").rolling(rolling).mean() / target_species
spec_dist_signal = (
    rd.select("species.distance").mean().rolling(rolling).mean() / target_species
)
spec_thresh_signal = rd.select("species.threshold").rolling(rolling).mean()
spec_evenness_signal = rd.select("species.evenness").rolling(rolling).mean()

distance_signal = (
    (rd.lit(0.9) * spec_count_signal)
    + (rd.lit(0.4) * spec_dist_signal)
    + (rd.lit(0.2) * spec_thresh_signal)
    + (rd.lit(0.1) * spec_evenness_signal)
).clamp(0.01, 10.0)


print(distance_signal.__repr__())

collector = rd.MetricCollector()

engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    .regression(inputs, answers, loss=rd.MSE)
    .subscribe(collector)
    .metrics(distance_signal=distance_signal)
    .diversity(rd.NeatDistance(), distance_signal)
    .alters(
        rd.Cross.graph(0.05, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)

result = engine.run(log=True, ui=True)

eval_results = result.value().eval(inputs)
accuracy = rd.accuracy(result.value(), inputs, answers, loss=rd.MSE)

print(result)
print(result.metrics().dashboard())
print(accuracy)

collector.plot(
    "species.threshold", "count.species", "rate.diversity", "species.evenness"
)
