import radiate as rd


# Setup (not shown): single-objective fitness for this snippet.
def single_fit(x):
    return 0.0


# --8<-- [start:single_objective]
import radiate as rd

# Create an engine. Float Scalar engine (one chromosome, with one gene)
single_obj_engine: rd.Engine[float, float] = (
    rd.Engine.float(init_range=(0.0, 1.0)).fitness(single_fit)
    # ... other parameters ...
)


# Run the engine for 100 generations
result: rd.Generation[float, float] = single_obj_engine.run(rd.Limit.generations(100))

# Get the best individual's decoded value
value: float = result.value()

# Get the score (fitness) of the best individual or epoch score
score: list[float] = result.score()  # note that this is a list.
# In this scenario, the engine is configured for single-objective optimization,
# so the list will contain a single value.

# Get the population of the engine's ecosystem
population: rd.Population[float] = result.population()  # Population object

# Get the index of the epoch (number of generations)
index: int = result.index()  # int

# Get the metrics of the engine
metrics: rd.MetricSet = result.metrics()  # MetricSet object

# Get the objective of the engine
objective: list[str] | str = (
    result.objective()
)  # list[str] | str (list[str] if multi-objective) - "min" or "max"
# --8<-- [end:single_objective]
