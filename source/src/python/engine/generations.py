# NOTE: the single- and multi-objective snippets use distinct variable names
# (`single_*` / `multi_*`) on purpose — they share a module scope here, and a static
# type checker rejects re-declaring the same name (e.g. `result`) with different generics.

import radiate as rd


# Setup (not shown): single-objective fitness for this snippet.
def single_fit(x):
    return 0.0


# --8<-- [start:single_objective]
import radiate as rd

# Create an engine. Float Scalar engine (one chromosome, with one gene)
single_obj_engine: rd.Engine[float, float] = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(single_fit)
    .limit(rd.Limit.generations(100))
    # ... other parameters ...
)


# Run the engine for 100 generations
single_result: rd.Generation[float, float] = single_obj_engine.run()

# Get the best individual's decoded value
value: float = single_result.value()

# Get the score (fitness) of the best individual or epoch score
score: list[float] = single_result.score()  # note that this is a list.
# In this scenario, the engine is configured for single-objective optimization,
# so the list will contain a single value.

# Get the population of the engine's ecosystem
population: rd.Population[float] = single_result.population()  # Population object

# Get the index of the epoch (number of generations)
index: int = single_result.index()  # int

# Get the metrics of the engine
metrics: rd.MetricSet = single_result.metrics()  # MetricSet object

# Get the objective of the engine
objective: list[str] | str = (
    single_result.objective()
)  # list[str] | str (list[str] if multi-objective) - "min" or "max"
# --8<-- [end:single_objective]


# Setup (not shown): multi-objective fitness for this snippet.
def multi_fit(x):
    return [0.0, 0.0]


import numpy as np

# --8<-- [start:multi_objective]
import radiate as rd

# Create an engine
multi_obj_engine: rd.Engine[float, list[np.ndarray]] = (
    rd.Engine.float(shape=[2, 2, 2], init_range=(0.0, 1.0), use_numpy=True)
    .fitness(multi_fit)  # Multi-objective fitness function
    .objective(rd.MIN, rd.MAX)  # Specify multi-objective optimization
    .limit(rd.Limit.generations(100))
    # ... other parameters ...
)

# Run the engine for 100 generations
multi_result: rd.Generation[float, list[np.ndarray]] = multi_obj_engine.run()

# Everything in the multi-objective epoch is the same as the single-objective epoch, except for the value.
# The function call to `front()` will return a `ParetoFront` object while `value()` will return None.:
front: rd.Front[float] = multi_result.front()  # ParetoFront object

# This is of type `Front` with `FrontValue` members.
value_at_index_0: rd.FrontValue[float] = front[0]  # FrontValue object
all_values: list[rd.FrontValue[float]] = front.values()  # list[FrontValue]

# Get the members of the Pareto front:
score: list[float] = all_values[0].score()  # list[float] - multi-objective score
genotype: rd.Genotype[float] = all_values[0].genotype()  # Genotype object
# --8<-- [end:multi_objective]
