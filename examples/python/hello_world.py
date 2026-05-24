#!/usr/bin/env python3
"""
Radiate example: Hello World

This is a simple hello world example that demonstrates a CharCodec to evolve a string of characters to match
a target string. The fitness function maximizes the number of characters in the genome that match
the target string.
"""

import radiate as rd

target = "Hello, Radiate!"


def fit(x: list[str]) -> int:
    return sum(1 for i in range(len(target)) if x[i] == target[i])


engine = (
    rd.Engine.char(len(target))
    .fitness(fit)
    .select(
        offspring=rd.Select.boltzmann(4.0)
    )  # <- specifying a selector is not really needed for such a simple problem, just for example purposes
    .limit(rd.Limit.score(len(target)))
)


result = engine.run(log=True)

print(result.metrics().dashboard())
print("Best solution:", "".join(result.value()))
