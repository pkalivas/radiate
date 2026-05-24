import radiate as rd


# Setup (not shown): multi-objective fitness for this snippet.
def multi_fit(x):
    return [0.0, 0.0]


# --8<-- [start:multi_objective]
import radiate as rd
import numpy as np

# Create an engine
multi_obj_engine: rd.Engine[float, list[np.ndarray]] = (
    rd.Engine.float(shape=[2, 2, 2], init_range=(0.0, 1.0), use_numpy=True)
    .fitness(multi_fit)  # Multi-objective fitness function
    .objective(rd.MIN, rd.MAX)  # Specify multi-objective optimization
    # ... other parameters ...
)

# Run the engine for 100 generations
result: rd.Generation[float, list[np.ndarray]] = multi_obj_engine.run(
    rd.Limit.generations(100)
)

# Everything in the multi-objective epoch is the same as the single-objective epoch, except for the value.
# The function call to `front()` will return a `ParetoFront` object while `value()` will return None.:
front: rd.Front[float] = result.front()  # ParetoFront object
# This is of type `Front` with `FrontValue` members.
value_at_index_0: rd.FrontValue[float] = front[0]  # FrontValue object
all_values: list[rd.FrontValue[float]] = front.values()  # list[FrontValue]

# Get the members of the Pareto front:
score: list[float] = all_values[0].score()  # list[float] - multi-objective score
genotype: rd.Genotype[float] = all_values[0].genotype()  # Genotype object
# --8<-- [end:multi_objective]
